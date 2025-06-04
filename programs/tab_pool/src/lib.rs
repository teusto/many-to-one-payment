use anchor_lang::prelude::*;

declare_id!("Hvddzj6Z3eFJheuabJGQNuchCoo4LjwDAQizRVTqW31D");

#[program]
mod tab_pool {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}