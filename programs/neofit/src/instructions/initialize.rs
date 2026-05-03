use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize {}

pub fn handler(_ctx: Context<Initialize>) -> Result<()> {
    msg!("This is a remnant of the past... This instruction is not supposed to be used.");
    Ok(())
}
