#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

#[program]
pub mod vesting {
    use super::*;

    pub fn initializevesting(ctx: Context<InitializeVesting>, company_name: String)-> Result<()>{
      Ok(())
    } 
}

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct InitializeVesting<'info>{
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(
    init,
    payer = signer,
    space = 8 + VestingAccount::InitSpace,
    seeds = [company_name.as_ref()],
    bump,
  )]
  pub vesting_account: Account<'info, VestingAccount>


}

#[account]
#[derive(InitSpace)]
pub struct VestingAccount {
  pub owner: Pubkey,
  pub mint_account: Pubkey,
  pub tresury_account: Pubkey,
  #[max_len(90)]
  pub company_name: String,
  pub tresury_bump: u8,
  pub bump: u8,
}


