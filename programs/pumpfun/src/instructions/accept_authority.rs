use constants::CONFIG;
use errors::PumpfunError;

use crate::*;

#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    //  Pending admin
    #[account(
        mut,
        constraint = global_config.pending_authority == new_admin.key() @PumpfunError::IncorrectAuthority
    )]
    pub new_admin: Signer<'info>,

    //  Stores admin address
    #[account(
        mut,
        seeds = [CONFIG.as_bytes()],
        bump,
    )]
    global_config: Box<Account<'info, Config>>,
}

impl AcceptAuthority<'_> {
    pub fn process_instruction(
        ctx: Context<Self>
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_config;
    
        global_state.authority = ctx.accounts.new_admin.key();
        global_state.pending_authority = Pubkey::default();
        
        Ok(())
    }
}