use crate::*;
use std::ops::{Div, Mul};
use anchor_spl::token;
use solana_program::program::{invoke, invoke_signed};

pub fn convert_to_float(value: u64, decimals: u8) -> f64 {
    (value as f64).div(f64::powf(10.0, decimals as f64))
}

pub fn convert_from_float(value: f64, decimals: u8) -> u64 {
    value.mul(f64::powf(10.0, decimals as f64)) as u64
}

//  transfer sol from user
pub fn sol_transfer_user<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    amount: u64,
) -> Result<()> {
    let ix = solana_program::system_instruction::transfer(source.key, destination.key, amount);
    invoke(&ix, &[source, destination, system_program])?;
    Ok(())
}

// transfer sol from PDA
pub fn sol_transfer_with_signer<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    signers: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let ix = solana_program::system_instruction::transfer(source.key, destination.key, amount);
    invoke_signed(&ix, &[source, destination, system_program], signers)?;
    Ok(())
}

//  transfer token from user
pub fn token_transfer_user<'a>(
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    to: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    amount: u64,
) -> Result<()> {
    let cpi_ctx: CpiContext<_> = CpiContext::new(
        token_program,
        token::Transfer {
            from,
            authority,
            to,
        },
    );
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}

//  transfer token from PDA
pub fn token_transfer_with_signer<'a>(
    from: AccountInfo<'a>,
    authority: AccountInfo<'a>,
    to: AccountInfo<'a>,
    token_program: AccountInfo<'a>,
    signers: &[&[&[u8]]],
    amount: u64,
) -> Result<()> {
    let cpi_ctx: CpiContext<_> = CpiContext::new_with_signer(
        token_program,
        token::Transfer {
            from,
            authority,
            to,
        },
        signers,
    );
    token::transfer(cpi_ctx, amount)?;

    Ok(())
}
