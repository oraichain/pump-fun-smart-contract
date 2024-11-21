use crate::{
    constants::{BONDING_CURVE, CONFIG, GLOBAL, METADATA},
    errors::*,
    state::{BondingCurve, Config},
};
use anchor_lang::{prelude::*, solana_program::sysvar::SysvarId, system_program};
use anchor_spl::{
    associated_token::{self, AssociatedToken}, 
    metadata::{self, mpl_token_metadata::types::DataV2, Metadata},
    token::{self, spl_token::instruction::AuthorityType, Mint, Token},
};

#[derive(Accounts)]
#[instruction(decimals: u8)]
pub struct Launch<'info> {
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
    
    #[account(mut)]
    creator: Signer<'info>,

    #[account(
        init,
        payer = creator,
        mint::decimals = decimals,
        mint::authority = global_vault.key(),
    )]
    token: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = creator,
        space = 8 + std::mem::size_of::<BondingCurve>(),
        seeds = [BONDING_CURVE.as_bytes(), &token.key().to_bytes()],
        bump
    )]
    bonding_curve: Box<Account<'info, BondingCurve>>,

    /// CHECK: passed to token metadata program
    #[account(
        mut,
        seeds = [
            METADATA.as_bytes(),
            metadata::ID.as_ref(),
            token.key().as_ref(),
        ],
        bump,
        seeds::program = metadata::ID
    )]
    token_metadata_account: UncheckedAccount<'info>,

    /// CHECK: created in instruction
    #[account(
        mut,
        seeds = [
            global_vault.key().as_ref(),
            token::spl_token::ID.as_ref(),
            token.key().as_ref(),
        ],
        bump,
        seeds::program = associated_token::ID
    )]
    global_token_account: UncheckedAccount<'info>,

    #[account(address = system_program::ID)]
    system_program: Program<'info, System>,

    #[account(address = Rent::id())]
    rent: Sysvar<'info, Rent>,

    #[account(address = token::ID)]
    token_program: Program<'info, Token>,

    #[account(address = associated_token::ID)]
    associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = metadata::ID)]
    mpl_token_metadata_program: Program<'info, Metadata>,
}

pub fn launch<'info>(
    ctx: Context<'_, '_, '_, 'info, Launch<'info>>,

    // launch config
    decimals: u8,
    token_supply: u64,
    reserve_lamport: u64,
    
    // metadata
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let global_config = &ctx.accounts.global_config;
    let creator = &ctx.accounts.creator;
    let token = &ctx.accounts.token;
    let global_token_account = &ctx.accounts.global_token_account;
    let bonding_curve = &mut ctx.accounts.bonding_curve;
    let global_vault = &ctx.accounts.global_vault;


    //  check params
    let decimal_multiplier = 10u64.pow(decimals as u32);
    let fractional_tokens = token_supply % decimal_multiplier;
    if fractional_tokens != 0 {
        msg!("expected whole number of tokens, got fractional tokens: 0.{fractional_tokens}");
        return Err(ValueInvalid.into());
    }

    global_config.lamport_amount_config
        .validate(&reserve_lamport)?;

    global_config.token_supply_config
        .validate(&(token_supply / decimal_multiplier))?;

    global_config.token_decimals_config
        .validate(&decimals)?;
    

    // create token launch pda
    bonding_curve.token_mint = token.key();
    bonding_curve.creator = creator.key();
    bonding_curve.init_lamport = reserve_lamport;
    bonding_curve.reserve_lamport = reserve_lamport;
    bonding_curve.reserve_token = token_supply;

    // create global token account
    associated_token::create(CpiContext::new(
        ctx.accounts.associated_token_program.to_account_info(),
        associated_token::Create {
            payer: creator.to_account_info(),
            associated_token: global_token_account.to_account_info(),
            authority: global_vault.to_account_info(),
            mint: token.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info()
        },
    ))?;

    let signer_seeds: &[&[&[u8]]] = &[&[
        GLOBAL.as_bytes(),
        &[ctx.bumps.global_vault],
    ]];

    // mint tokens
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                mint: token.to_account_info(),
                to: global_token_account.to_account_info(),
                authority: global_vault.to_account_info(),
            },
            signer_seeds,
        ),
        token_supply,
    )?;

    // create metadata
    metadata::create_metadata_accounts_v3(
        CpiContext::new_with_signer(
            ctx.accounts.mpl_token_metadata_program.to_account_info(),
            metadata::CreateMetadataAccountsV3 {
                metadata: ctx.accounts.token_metadata_account.to_account_info(),
                mint: token.to_account_info(),
                mint_authority: global_vault.to_account_info(),
                payer: creator.to_account_info(),
                update_authority: global_vault.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
            signer_seeds,
        ),
        DataV2 {
            name,
            symbol,
            uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        },
        false,
        true,
        None,
    )?;

    //  revoke mint authority
    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::SetAuthority {
                current_authority: global_vault.to_account_info(),
                account_or_mint: token.to_account_info(),
            },
            signer_seeds,
        ),
        AuthorityType::MintTokens,
        None,
    )?;

    bonding_curve.is_completed = false;

    Ok(())
}
