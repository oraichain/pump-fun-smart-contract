use anchor_lang::{system_program, prelude::*};
use anchor_spl::{
    associated_token::{self, AssociatedToken},
    token::{self, Mint, Token},
};
use crate::{
    constants::{BONDING_CURVE, CONFIG, GLOBAL}, errors::PumpfunError, state::{BondingCurve, Config, BondingCurveAccount}
};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,
    
    //  team wallet
    /// CHECK: should be same with the address in the global_config
    #[account(
        mut,
        constraint = global_config.team_wallet == team_wallet.key() @PumpfunError::IncorrectAuthority
    )]
    pub team_wallet: AccountInfo<'info>,

    /// CHECK: ata of team wallet
    #[account(
        mut,
        seeds = [
            team_wallet.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    team_wallet_ata: AccountInfo<'info>,


    #[account(
        mut,
        seeds = [BONDING_CURVE.as_bytes(), &token_mint.key().to_bytes()], 
        bump
    )]
    bonding_curve: Account<'info, BondingCurve>,

    /// CHECK: global vault pda which stores SOL
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    pub token_mint: Box<Account<'info, Mint>>,

    /// CHECK: ata of global vault
    #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    global_ata: AccountInfo<'info>,

    /// CHECK: ata of user
    #[account(
        mut,
        seeds = [
            user.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    user_ata: AccountInfo<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = token::ID)]
    pub token_program: Program<'info, Token>,

    #[account(address = associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn swap(ctx: Context<Swap>, amount: u64, direction: u8, minimum_receive_amount: u64) -> Result<u64> {
    let bonding_curve = &mut ctx.accounts.bonding_curve;

    //  check curve is not completed

    let token_one_accounts = (
        &mut *ctx.accounts.token_mint.clone(),
        &mut ctx.accounts.global_ata.to_account_info(),
        &mut ctx.accounts.user_ata.to_account_info(),
    );

    let token_two_accounts = (
        &mut ctx.accounts.global_vault.to_account_info(),
        &mut ctx.accounts.user.to_account_info()
    );

    let team_wallet_accounts = (
        &mut ctx.accounts.team_wallet.to_account_info(),
        &mut ctx.accounts.team_wallet_ata.to_account_info()
    );

    let token = &mut ctx.accounts.token_mint;
    let team_wallet = &mut ctx.accounts.team_wallet;
    let team_wallet_ata = &mut ctx.accounts.team_wallet_ata;
    let user_ata = &mut ctx.accounts.user_ata;

    //  create user wallet ata, if it doean't exit
    if user_ata.data_is_empty() {
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.user.to_account_info(),
                associated_token: user_ata.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),

                mint: token.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            }
        ))?;
    }

    //  create team wallet ata, if it doesn't exist
    if team_wallet_ata.data_is_empty() {
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.user.to_account_info(),
                associated_token: team_wallet_ata.to_account_info(),
                authority: team_wallet.to_account_info(),

                mint: token.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            }
        ))?;
    }

    let signer_seeds: &[&[&[u8]]] = &[&[
        GLOBAL.as_bytes(),
        &[ctx.bumps.global_vault],
    ]];

    let amount_out = bonding_curve.swap(
        &*ctx.accounts.global_config,
        token_one_accounts,
        token_two_accounts,
        team_wallet_accounts,

        amount,
        direction,
        minimum_receive_amount,

        &ctx.accounts.user,
        signer_seeds,

        &ctx.accounts.token_program,
        &ctx.accounts.system_program,
    )?;
    
    Ok(amount_out)
}
