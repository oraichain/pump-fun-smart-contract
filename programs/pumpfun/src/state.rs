use crate::errors::*;
use crate::events::CompleteEvent;
use crate::utils::*;
use anchor_lang::{prelude::*, AnchorDeserialize, AnchorSerialize};
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use core::fmt::Debug;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

#[account]
pub struct Config {
    pub authority: Pubkey,
    //  use this for 2 step ownership transfer
    pub pending_authority: Pubkey,

    pub team_wallet: Pubkey,

    pub platform_buy_fee: f64,  //  platform fee percentage
    pub platform_sell_fee: f64,

    pub curve_limit: u64,       //  lamports to complete te bonding curve

    pub lamport_amount_config: AmountConfig<u64>,
    pub token_supply_config: AmountConfig<u64>,
    pub token_decimals_config: AmountConfig<u8>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AmountConfig<T: PartialEq + PartialOrd + Debug> {
    Range { min: Option<T>, max: Option<T> },
    Enum(Vec<T>),
}

impl<T: PartialEq + PartialOrd + Debug> AmountConfig<T> {
    pub fn validate(&self, value: &T) -> Result<()> {
        match self {
            Self::Range { min, max } => {
                if let Some(min) = min {
                    if value < min {
                        msg!("value {value:?} too small, expected at least {min:?}");
                        return Err(ValueTooSmall.into());
                    }
                }
                if let Some(max) = max {
                    if value > max {
                        msg!("value {value:?} too large, expected at most {max:?}");
                        return Err(ValueTooLarge.into());
                    }
                }

                Ok(())
            }
            Self::Enum(options) => {
                if options.contains(value) {
                    Ok(())
                } else {
                    msg!("invalid value {value:?}, expected one of: {options:?}");
                    Err(ValueInvalid.into())
                }
            }
        }
    }
}

#[account]
pub struct BondingCurve {
    pub token_mint: Pubkey,
    pub creator: Pubkey,

    pub init_lamport: u64,

    pub reserve_lamport: u64,
    pub reserve_token: u64,

    pub is_completed: bool,
}
pub trait BondingCurveAccount<'info> {
    // Updates the token reserves in the liquidity pool
    fn update_reserves(&mut self, global_config: &Account<'info, Config>, reserve_one: u64, reserve_two: u64) -> Result<bool>;

    fn swap(
        &mut self,
        global_config: &Account<'info, Config>,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut AccountInfo<'info>,
            &mut AccountInfo<'info>,
        ),
        token_two_accounts: (&mut AccountInfo<'info>, &mut AccountInfo<'info>),
        team_wallet_accounts: (&mut AccountInfo<'info>, &mut AccountInfo<'info>),
        amount: u64,
        direction: u8,

        user: &Signer<'info>,
        signer: &[&[&[u8]]],

        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()>;
}

impl<'info> BondingCurveAccount<'info> for Account<'info, BondingCurve> {
    fn update_reserves(&mut self, global_config: &Account<'info, Config>, reserve_token: u64, reserve_lamport: u64) -> Result<bool> {
        self.reserve_token = reserve_token;
        self.reserve_lamport = reserve_lamport;

        if reserve_lamport >= global_config.curve_limit {
            msg!("curve is completed");
            self.is_completed = true;
            return Ok(true)
        }

        Ok(false)
    }

    fn swap(
        &mut self,
        global_config: &Account<'info, Config>,
        token_one_accounts: (
            &mut Account<'info, Mint>,
            &mut AccountInfo<'info>,
            &mut AccountInfo<'info>,
        ),
        token_two_accounts: (&mut AccountInfo<'info>, &mut AccountInfo<'info>),
        team_wallet_accounts: (&mut AccountInfo<'info>, &mut AccountInfo<'info>),

        amount: u64,
        direction: u8,

        user: &Signer<'info>,
        signer: &[&[&[u8]]],

        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        if amount <= 0 {
            return err!(PumpfunError::InvalidAmount);
        }
        msg!("Mint: {:?} ", token_one_accounts.0.key());
        msg!("Swap: {:?} {:?} {:?}", user.key(), direction, amount);

        // xy = k => Constant product formula
        // (x + dx)(y - dy) = k
        // y - dy = k / (x + dx)
        // y - dy = xy / (x + dx)
        // dy = y - (xy / (x + dx))
        // dy = yx + ydx - xy / (x + dx)
        // formula => dy = ydx / (x + dx)

        let fee_percent = if direction == 1 {
            global_config.platform_sell_fee
        } else {
            global_config.platform_buy_fee
        };

        let adjusted_amount_in_float = convert_to_float(amount, token_one_accounts.0.decimals)
            .div(100_f64)
            .mul(100_f64.sub(fee_percent));

        let adjusted_amount =
            convert_from_float(adjusted_amount_in_float, token_one_accounts.0.decimals);

        if direction == 1 {
            let denominator_sum = self
                .reserve_token
                .checked_add(adjusted_amount)
                .ok_or(PumpfunError::OverflowOrUnderflowOccurred)?;

            let div_amt = convert_to_float(denominator_sum, token_one_accounts.0.decimals).div(
                convert_to_float(adjusted_amount, token_one_accounts.0.decimals),
            );

            let amount_out_in_float = convert_to_float(self.reserve_lamport, 9 as u8).div(div_amt);

            let amount_out = convert_from_float(amount_out_in_float, 9 as u8);

            let new_reserves_one = self
                .reserve_token
                .checked_add(amount)
                .ok_or(PumpfunError::OverflowOrUnderflowOccurred)?;

            let new_reserves_two = self
                .reserve_lamport
                .checked_sub(amount_out)
                .ok_or(PumpfunError::OverflowOrUnderflowOccurred)?;

            self.update_reserves(global_config, new_reserves_one, new_reserves_two)?;
            msg! {"Reserves: {:?} {:?}", new_reserves_one, new_reserves_two}
            token_transfer_user(
                token_one_accounts.2.to_account_info().clone(),
                user.to_account_info().clone(),
                token_one_accounts.1.to_account_info().clone(),
                token_program.to_account_info(),
                adjusted_amount,
            )?;

            sol_transfer_with_signer(
                token_two_accounts.0.to_account_info().clone(),
                token_two_accounts.1.to_account_info().clone(),
                system_program.to_account_info(),
                signer,
                amount_out
            )?;

            //  transfer fee to team wallet
            let fee_amount = amount - adjusted_amount;

            msg! {"fee: {:?}", fee_amount}

            token_transfer_user(
                token_one_accounts.2.to_account_info().clone(),
                user.to_account_info().clone(),
                team_wallet_accounts.1.to_account_info().clone(),
                token_program.to_account_info(),
                fee_amount
            )?;
            
        } else {
            let denominator_sum = self
                .reserve_lamport
                .checked_add(adjusted_amount)
                .ok_or(PumpfunError::OverflowOrUnderflowOccurred)?;

            let div_amt = convert_to_float(denominator_sum, token_one_accounts.0.decimals).div(
                convert_to_float(adjusted_amount, token_one_accounts.0.decimals),
            );

            let amount_out_in_float = convert_to_float(self.reserve_token, 9 as u8).div(div_amt);

            let amount_out = convert_from_float(amount_out_in_float, 9 as u8);

            let new_reserves_one = self
                .reserve_token
                .checked_sub(amount_out)
                .ok_or(PumpfunError::OverflowOrUnderflowOccurred)?;

            let new_reserves_two = self
                .reserve_lamport
                .checked_add(amount)
                .ok_or(PumpfunError::OverflowOrUnderflowOccurred)?;

            let is_completed = self.update_reserves(global_config, new_reserves_one, new_reserves_two)?;

            if is_completed == true {
                emit!(
                    CompleteEvent {
                        user: token_two_accounts.1.key(), 
                        mint: token_one_accounts.0.key(), 
                        bonding_curve: self.key()
                    }
                );
            }

            msg! {"Reserves: {:?} {:?}", new_reserves_one, new_reserves_two}

            token_transfer_with_signer(
                token_one_accounts.1.to_account_info().clone(),
                token_two_accounts.0.to_account_info().clone(),
                token_one_accounts.2.to_account_info().clone(),
                token_program.to_account_info(),
                signer,
                amount_out,
            )?;

            sol_transfer_user(
                token_two_accounts.1.to_account_info().clone(),
                token_two_accounts.0.to_account_info().clone(),
                system_program.to_account_info(),
                amount
            )?;
            
            //  transfer fee to team wallet, pegasus wallet
            let fee_amount = amount - adjusted_amount;

            sol_transfer_user(
                token_two_accounts.1.to_account_info().clone(),
                team_wallet_accounts.0.to_account_info().clone(),
                system_program.to_account_info(),
                fee_amount
            )?;
        }
        Ok(())
    }

}
