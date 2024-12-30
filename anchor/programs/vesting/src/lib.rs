#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

//creating a smartcontract program
#[program]
pub mod vesting {
  use super::*;

  pub fn create_vesting(ctx: Context<CreateVestingAccount>, company_name: String) -> Result<()>{

    Ok(())
  }
}

#[derive(Accounts)]
//providing company_name
#[instruction(company_name: String)]
pub struct CreateVestingAccount<'info>{
  #[account(mut)]
  pub signer: Signer<'info>,

  //initializing a new account 
  #[account(
     init,
     payer = signer,
    
     space = 8 + VestingAccount::INIT_SPACE,
      //using campany_name for making each pda inique so anytime we can derive a specific pda using it 
     seeds = [company_name.as_ref()],
     bump,
  )]
  pub vesting_account: Account<'info, VestingAccount>,
  //creating a mint account to mint spl-tokens
  pub mint: Account<'info, Mint>,

  //this is token-account not a ata as we are proving seeds to derive it later
  #[account(
    init,
    token::mint = mint,
    token::authority = treasury_token_account,
    payer = signer,
    //as here seeds is string
    seeds = [b"vesting_treasury", company_name.as_bytes()],
    bump
  )]
  pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,


  pub system_program: Program<'info,System>,
  pub token_program: Interface<'info, TokenInterface>

}

#[account]
#[derive(InitSpace)]
pub struct VestingAccount{
   pub owner: Pubkey,
   pub mint: Pubkey,
   pub treasury_token_account: Pubkey,
   #[max_len(32)]
   pub company_name: String,
   //this bump will make every pdas or account unique 
   pub treasury_bump: u8,
   pub bump: u8
}