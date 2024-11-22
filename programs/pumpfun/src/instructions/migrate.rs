use std::ops::{Div, Mul};

use anchor_lang::{prelude::*, solana_program::program::invoke_signed};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token},
};

use crate::{
    amm_instruction,
    constants::{BONDING_CURVE, CONFIG, GLOBAL},
    errors::PumpfunError,
    state::{BondingCurve, BondingCurveAccount, Config},
    utils::{convert_from_float, convert_to_float, sol_transfer_with_signer, token_transfer_with_signer},
};

#[derive(Accounts)]
pub struct Migrate<'info> {
    /// CHECK: Safe
    #[account(
        mut,
        constraint = global_config.team_wallet == *team_wallet.key @PumpfunError::IncorrectAuthority
    )]
    pub team_wallet: AccountInfo<'info>,

    #[account(
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [BONDING_CURVE.as_bytes(), &coin_mint.key().to_bytes()],
        bump
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    /// CHECK
    #[account(
        mut,
        seeds = [GLOBAL.as_bytes()],
        bump,
    )]
    pub global_vault: AccountInfo<'info>,

    /// CHECK: Safe
    pub amm_program: AccountInfo<'info>,
    /// CHECK: Safe. The spl token program
    pub token_program: Program<'info, Token>,
    /// CHECK: Safe. The associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// CHECK: Safe. System program
    pub system_program: Program<'info, System>,
    /// CHECK: Safe. Rent program
    pub sysvar_rent: Sysvar<'info, Rent>,
    /// CHECK: Safe.
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            market.key.as_ref(),
            b"amm_associated_seed"],
        bump,
        seeds::program = amm_program.key
    )]
    pub amm: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        seeds = [b"amm authority"],
        bump,
        seeds::program = amm_program.key
    )]
    pub amm_authority: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            market.key.as_ref(),
            b"open_order_associated_seed"],
        bump,
        seeds::program = amm_program.key
    )]
    pub amm_open_orders: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            market.key.as_ref(),
            b"lp_mint_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub lp_mint: AccountInfo<'info>,

    #[account(mut)]
    pub coin_mint: Box<Account<'info, Mint>>,
    /// CHECK: Safe. Pc mint account
    #[account(mut)]
    pub pc_mint: Box<Account<'info, Mint>>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            market.key.as_ref(),
            b"coin_vault_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub coin_vault: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            market.key.as_ref(),
            b"pc_vault_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub pc_vault: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [
            amm_program.key.as_ref(),
            market.key.as_ref(),
            b"target_associated_seed"
        ],
        bump,
        seeds::program = amm_program.key
    )]
    pub target_orders: AccountInfo<'info>,
    /// CHECK: Safe
    #[account(
        mut,
        seeds = [b"amm_config_account_seed"],
        bump,
        seeds::program = amm_program.key
    )]
    pub amm_config: AccountInfo<'info>,

    /// CHECK: Safe. OpenBook program.
    pub market_program: AccountInfo<'info>,
    /// CHECK: Safe. OpenBook market. OpenBook program is the owner.
    #[account(mut)]
    pub market: AccountInfo<'info>,
    /// CHECK: Safe. The user wallet create the pool
    #[account(mut)]
    pub user_wallet: Signer<'info>,

    /// CHECK: verified in transfer instruction
    #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            coin_mint.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    global_token_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            team_wallet.key().as_ref(),
            anchor_spl::token::spl_token::ID.as_ref(),
            coin_mint.key().as_ref(),
        ],
        bump,
        seeds::program = anchor_spl::associated_token::ID
    )]
    team_ata: AccountInfo<'info>,

    /// CHECK: Safe. The user pc token
    #[account(mut)]
    pub user_token_pc: AccountInfo<'info>,

    /// CHECK: Safe. The user lp token
    #[account(mut)]
    pub user_token_lp: AccountInfo<'info>,
}

pub fn migrate(ctx: Context<Migrate>, nonce: u8) -> Result<()> {
    let global_config = &mut ctx.accounts.global_config;
    let bonding_curve = &mut ctx.accounts.bonding_curve;

    //  check curve is completed
    require!(
        bonding_curve.is_completed == true,
        PumpfunError::CurveNotCompleted
    );

    //  check mint, freeze authorities are revoked
    require!(
        ctx.accounts.coin_mint.freeze_authority.is_none(),
        PumpfunError::FreezeAuthorityEnabled
    );
    require!(
        ctx.accounts.coin_mint.mint_authority.is_none(),
        PumpfunError::MintAuthorityEnabled
    );

    let lamport_on_curve = bonding_curve.reserve_lamport - bonding_curve.init_lamport;

    let fee_in_float = convert_to_float(lamport_on_curve, ctx.accounts.coin_mint.decimals)
        .div(100_f64)
        .mul(global_config.platform_migration_fee);

    let fee_lamport = convert_from_float(fee_in_float, ctx.accounts.coin_mint.decimals);

    //  0.32 - market create
    //  0.4  - pool create
    //  0.01 - tx
    let init_pc_amount = lamport_on_curve - fee_lamport - 730_000_000;

    let coin_amount = (init_pc_amount as u128 * bonding_curve.reserve_token as u128
        / bonding_curve.reserve_lamport as u128) as u64;
    let fee_token = bonding_curve.reserve_token - coin_amount;

    msg!(
        "Curve:: Init {:?} Token: {:?}  Sol: {:?}",
        bonding_curve.init_lamport,
        bonding_curve.reserve_token,
        bonding_curve.reserve_lamport
    );
    
    msg!(
        "Fee:: Token: {:?}  Sol: {:?}",
        fee_lamport,
        fee_token
    );

    let signer_seeds: &[&[&[u8]]] = &[&[
        GLOBAL.as_bytes(),
        &[ctx.bumps.global_vault],
    ]];

    //  transfer 0.33 SOL to signer for market id creation fee + tx fee
    sol_transfer_with_signer(
        ctx.accounts.global_vault.to_account_info(),
        ctx.accounts.user_wallet.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        signer_seeds,
        330_000_000,
    )?;

    //  transfer migration fee to team wallet
    sol_transfer_with_signer(
        ctx.accounts.global_vault.to_account_info(),
        ctx.accounts.team_wallet.to_account_info(),
        ctx.accounts.system_program.to_account_info(),
        signer_seeds,
        fee_lamport
    )?;
    token_transfer_with_signer(
        ctx.accounts.global_token_account.to_account_info(),
        ctx.accounts.global_vault.to_account_info(),
        ctx.accounts.team_ata.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        signer_seeds,
        fee_token
    )?;

    //  Running raydium amm initialize2
    let initialize_ix = amm_instruction::initialize2(
        ctx.accounts.amm_program.key,
        ctx.accounts.amm.key,
        ctx.accounts.amm_authority.key,
        ctx.accounts.amm_open_orders.key,
        ctx.accounts.lp_mint.key,
        &ctx.accounts.coin_mint.key(),
        &ctx.accounts.pc_mint.key(),
        ctx.accounts.coin_vault.key,
        ctx.accounts.pc_vault.key,
        ctx.accounts.target_orders.key,
        ctx.accounts.amm_config.key,
        ctx.accounts.team_wallet.key,
        ctx.accounts.market_program.key,
        ctx.accounts.market.key,
        //  change this to PDA address
        ctx.accounts.global_vault.key,
        ctx.accounts.global_token_account.key,
        ctx.accounts.user_token_pc.key,
        &ctx.accounts.user_token_lp.key(),
        nonce,
        Clock::get()?.unix_timestamp as u64,
        init_pc_amount,
        coin_amount,
    )?;
    let account_infos = [
        ctx.accounts.amm_program.clone(),
        ctx.accounts.amm.clone(),
        ctx.accounts.amm_authority.clone(),
        ctx.accounts.amm_open_orders.clone(),
        ctx.accounts.lp_mint.clone(),
        ctx.accounts.coin_mint.to_account_info().clone(),
        ctx.accounts.pc_mint.to_account_info().clone(),
        ctx.accounts.coin_vault.clone(),
        ctx.accounts.pc_vault.clone(),
        ctx.accounts.target_orders.clone(),
        ctx.accounts.amm_config.clone(),
        ctx.accounts.team_wallet.clone(),
        ctx.accounts.market_program.clone(),
        ctx.accounts.market.clone(),
        ctx.accounts.global_vault.clone(),
        ctx.accounts.global_token_account.clone(),
        ctx.accounts.user_token_pc.clone(),
        ctx.accounts.user_token_lp.clone(),
        ctx.accounts.token_program.to_account_info().clone(),
        ctx.accounts.system_program.to_account_info().clone(),
        ctx.accounts
            .associated_token_program
            .to_account_info()
            .clone(),
        ctx.accounts.sysvar_rent.to_account_info().clone(),
    ];
    invoke_signed(&initialize_ix, &account_infos, signer_seeds)?;

    msg!(
        "Raydium Input:: Token: {:?}  Sol: {:?}",
        coin_amount,
        init_pc_amount
    );

    //  update reserves
    bonding_curve.update_reserves(&*ctx.accounts.global_config, 0, 0)?;

    Ok(())
}
