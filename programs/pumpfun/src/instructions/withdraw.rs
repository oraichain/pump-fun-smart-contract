use crate::{
    constants::{BONDING_CURVE, CONFIG, GLOBAL}, errors::*, state::{BondingCurve, Config, BondingCurveAccount}, utils::{sol_transfer_with_signer, token_transfer_with_signer}
};
use anchor_lang::{prelude::*, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken}, 
    token::{self, Mint, Token},
};

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    /// CHECK: global vault pda which stores SOL
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,
    
    #[account(
        mut,
        constraint = global_config.authority == admin.key() @PumpfunError::IncorrectAuthority
    )]
    admin: Signer<'info>,

    token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [BONDING_CURVE.as_bytes(), &token_mint.key().to_bytes()], 
        bump
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

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
    global_vault_ata: AccountInfo<'info>,

    /// CHECK: ata of admin
    #[account(
        mut,
        seeds = [
            admin.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    admin_ata: AccountInfo<'info>,

    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,

    #[account(address = token::ID)]
    token_program: Program<'info, Token>,

    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,
}

pub fn withdraw<'info>(
    ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>,
) -> Result<()> {
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let global_config = &mut ctx.accounts.global_config;
    let admin_ata = &mut ctx.accounts.admin_ata;

    require!(bonding_curve.is_completed == true, PumpfunError::CurveNotCompleted);

    bonding_curve.update_reserves(global_config, 0, 0)?;

    //  create admin wallet ata, if it doesn't exist
    if admin_ata.data_is_empty() {
        anchor_spl::associated_token::create(CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: ctx.accounts.admin.to_account_info(),
                associated_token: admin_ata.to_account_info(),
                authority: ctx.accounts.admin.to_account_info(),

                mint: ctx.accounts.token_mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            }
        ))?;
    }

    // transfer sol/token to admin wallet
    let lamport_amount = bonding_curve.reserve_lamport - bonding_curve.init_lamport;
    let signer_seeds: &[&[&[u8]]] = &[&[
        GLOBAL.as_bytes(),
        &[ctx.bumps.global_vault],
    ]];

    sol_transfer_with_signer(
        ctx.accounts.global_vault.to_account_info(),
        ctx.accounts.admin.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        signer_seeds,
        lamport_amount,
    )?;

    token_transfer_with_signer(
        ctx.accounts.global_vault_ata.to_account_info(),
        ctx.accounts.global_vault.to_account_info(),
        ctx.accounts.admin_ata.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        signer_seeds,
        bonding_curve.reserve_token,
    )?;

    Ok(())
}
