use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{
    // errors::CustomError,
    state::{LiquidityPool, LiquidityPoolAccount, LiquidityProvider},
};

pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_one: u64, amount_two: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    // (token, to, from)
    let token_one_accounts = (
        &mut *ctx.accounts.mint_token_one.clone(),
        &mut *ctx.accounts.pool_token_account_one,
        &mut *ctx.accounts.user_token_account_one,
    );

    // let token_two_accounts = (
    //     &mut *ctx.accounts.mint_token_one.clone(),
    //     &mut pool.to_account_info().clone(),
    //     &mut ctx.accounts.user.to_account_info().clone(),
    // );

    pool.set_inner(LiquidityPool::new(
        ctx.accounts.mint_token_one.key(),
        ctx.bumps.pool,
    ));

    // amount two is sol amount
    pool.add_liquidity(
        token_one_accounts,
        // token_two_accounts,
        amount_one,
        amount_two,
        &mut *ctx.accounts.liquidity_provider_account,
        &ctx.accounts.user,
        &ctx.accounts.token_program,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        init,
        space = LiquidityPool::ACCOUNT_SIZE,
        payer = user,
        seeds = [LiquidityPool::POOL_SEED_PREFIX.as_bytes(), mint_token_one.key().as_ref()], // each mint_token refer to one pool account
        bump
    )]
    pub pool: Box<Account<'info, LiquidityPool>>,

    #[account(
        init_if_needed,
        payer = user,
        space = LiquidityProvider::ACCOUNT_SIZE,
        seeds = [LiquidityProvider::SEED_PREFIX.as_bytes(), pool.key().as_ref(), user.key().as_ref()],
        bump,
    )]
    pub liquidity_provider_account: Box<Account<'info, LiquidityProvider>>,

    #[account(mut)]
    pub mint_token_one: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = user,
        associated_token::mint = mint_token_one,
        associated_token::authority = pool
    )]
    pub pool_token_account_one: Box<Account<'info, TokenAccount>>,

    // #[account(
    //     mut,
    //     associated_token::mint = mint_token_two,
    //     associated_token::authority = pool
    // )]
    // pub pool_token_account_two: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_token_one,
        associated_token::authority = user,
    )]
    pub user_token_account_one: Box<Account<'info, TokenAccount>>,

    // #[account(
    //     mut,
    //     associated_token::mint = mint_token_two,
    //     associated_token::authority = user,
    // )]
    // pub user_token_account_two: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
