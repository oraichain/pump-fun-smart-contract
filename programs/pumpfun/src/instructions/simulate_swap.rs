use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use crate::{
    constants::{BONDING_CURVE, CONFIG}, state::{BondingCurve, Config, BondingCurveAccount}
};

#[derive(Accounts)]
pub struct SimulateSwap<'info> {
    #[account(
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    #[account(
        seeds = [BONDING_CURVE.as_bytes(), &token_mint.key().to_bytes()], 
        bump
    )]
    bonding_curve: Account<'info, BondingCurve>,

    pub token_mint: Box<Account<'info, Mint>>,
}

pub fn simulate_swap(ctx: Context<SimulateSwap>, amount: u64, direction: u8) -> Result<u64> {
    let bonding_curve = &ctx.accounts.bonding_curve;

    let token_one_accounts = (
        &*ctx.accounts.token_mint.clone(),
    );

    let amount_out = bonding_curve.simulate_swap(
        &*ctx.accounts.global_config,
        token_one_accounts,

        amount,
        direction,
    )?;
    
    Ok(amount_out)
}
