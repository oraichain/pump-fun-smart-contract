
use constants::CONFIG;
use errors::PumpfunError;

use crate::*;

#[derive(Accounts)]
pub struct NominateAuthority<'info> {
    // Current admin
    #[account(
        mut,
        constraint = global_config.authority == *admin.key @PumpfunError::IncorrectAuthority
    )]
    pub admin: Signer<'info>,

    //  Stores admin address
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,
}

impl NominateAuthority<'_> {
    pub fn process_instruction(
        ctx: Context<Self>,
        new_admin: Pubkey
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_config;
    
        global_state.pending_authority = new_admin;
        Ok(())
    }
}
