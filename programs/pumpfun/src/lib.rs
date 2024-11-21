pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod utils;
pub mod events;

use crate::instructions::*;
use anchor_lang::prelude::*;
use state::Config;

declare_id!("wenBqrmxFAvovtz2jVRyNKjQQWzFF23Qv5oz3PSvDEW");

#[program]
pub mod pumpfun {
    use super::*;

    //  called by admin to set global config
    //  need to check the signer is authority
    pub fn configure<'info>(
        ctx: Context<'_, '_, '_, 'info, Configure<'info>>,
        new_config: Config,
    ) -> Result<()> {
        instructions::configure(ctx, new_config)
    }

    //  Admin can hand over admin role
    pub fn nominate_authority(ctx: Context<NominateAuthority>, new_admin: Pubkey) -> Result<()> {
        NominateAuthority::process_instruction(ctx, new_admin)
    }

    //  Pending admin should accept the admin role
    pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
        AcceptAuthority::process_instruction(ctx)
    }

    pub fn launch<'info>(
        ctx: Context<'_, '_, '_, 'info, Launch<'info>>,

        // launch config
        decimals: u8,
        token_supply: u64,
        virtual_lamport_reserves: u64,

        //  metadata
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::launch(
            ctx,
            decimals,
            token_supply,
            virtual_lamport_reserves,
            name,
            symbol,
            uri,
        )
    }

    //  amount - swap amount
    //  direction - 0: buy, 1: sell
    pub fn swap<'info>(
        ctx: Context<'_, '_, '_, 'info, Swap<'info>>,
        amount: u64,
        direction: u8
    ) -> Result<()> {
        instructions::swap(ctx, amount, direction)
    }
    
    //  admin can withdraw sol/token after the curve is completed
    //  backend receives a event when the curve is completed and call this instruction
    pub fn withdraw<'info>(
        ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>
    ) -> Result<()> {
        instructions::withdraw(ctx)
    }
}
