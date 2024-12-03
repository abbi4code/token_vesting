#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token, token_interface::{Mint, TokenAccount, TokenInterface}};

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

#[program]
pub mod vesting {
    use super::*;

    
    pub fn initializevesting(ctx: Context<InitializeVesting>, company_name: String)-> Result<()>{
      *ctx.accounts.vesting_account = VestingAccount{
        owner: ctx.accounts.signer.key(),
        mint_account: ctx.accounts.mint.key(),
        tresury_account:ctx.accounts.tresury_account.key(),
        company_name,
        tresury_bump: ctx.bumps.tresury_account,
        bump: ctx.bumps.vesting_account,
      };
      

      Ok(())
    }
    pub fn initialize_employee(ctx: Context<InitializeEmployee>, start_time: i64, end_time:i64, total_amount: u64, cliff_time: i64) -> Result<()> {
      *ctx.accounts.employee_account = EmployeeAccount {
        beneficiary: ctx.accounts.beneficiary.key(),
        vesting_account: ctx.accounts.vesting_account.key(),
        start_time,
        end_time,
        cliff_time,
        total_amount,
        total_withdrawn: 0,
        bump: ctx.bumps.employee_account,

      };
      Ok(())
    }

    pub fn claim_tokens(ctx: Context<ClaimTokens>, company_name: String) -> Result<()>{

       let employee_account = &mut ctx.accounts.employee_account;
       
       let curr_time = Clock::get()?.unix_timestamp;

      

      Ok(())
    }
}

#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct ClaimTokens<'info>{
  #[account(mut)]
  pub beneficiary: Signer<'info>,

  #[account(
    mut,
    seeds = [b"employee_vesting", beneficiary.key().as_ref(),vesting_account.key().as_ref()],
    bump = employee_account.bump,
    has_one = beneficiary,
    has_one = vesting_account
  )]
  pub employee_account: Account<'info,EmployeeAccount>,

  #[account(
    mut,
    seeds = [company_name.as_ref()],
    has_one = mint,
    has_one = tresury_account,
    bump = vesting_account.bump
  )]
  pub vesting_account: Account<'info, VestingAccount>,

  pub mint: InterfaceAccount<'info, Mint>,

  #[account(mut)]
  pub tresury_account: InterfaceAccount<'info, TokenAccount>,

  //now creating ATA for sending back these tokens into accoutns
  //we need the ATA contrains mint,authority
  // as there could some guys which already have a token_account(ATA)
  #[account(
    init_if_needed,,
    payer = beneficiary,
    associated_token::mint = mint,
    associated_token::authority = beneficiary,
    associated_token::token_program = token_program
  )]
  pub employee_token_account: InterfaceAccount<'info, TokenAccount>,

  pub token_program: Interface<'info, TokenInterface>,

  pub associated_token_program: Program<'info, AssociatedToken>,

  pub system_program: Program<'info, System>







}

#[derive(Accounts)]
pub struct InitializeEmployee<'info>{
  #[account(mut)]
  pub owner: Signer<'info>,
  pub beneficiary: SystemAccount<'info>,

  //making sure that owner of the vesting account is the signer of this instruction
  #[account(
    has_one = owner,
  )]
  pub vesting_account: Account<'info, VestingAccount>,
  #[account(
    init,
    payer = owner,
    space = 8 + EmployeeAccount::INIT_SPACE,
    seeds = [b"employee_vesting", beneficiary.key().as_ref(),vesting_account.key().as_ref()],
    bump
  )]
  pub employee_account: Account<'info, EmployeeAccount>,

  pub system_program: Program<'info, System>

}


#[derive(Accounts)]
#[instruction(company_name: String)]
pub struct InitializeVesting<'info>{
  #[account(mut)]
  pub signer: Signer<'info>,

  #[account(
    init,
    payer = signer,
    space = 8 + VestingAccount::INIT_SPACE,
    seeds = [company_name.as_ref()],
    bump,
  )]
  pub vesting_account: Account<'info, VestingAccount>,

  pub mint: InterfaceAccount<'info, Mint>,
  //this is account which will holds up token
  #[account(
    init,
    token::mint = mint,
    token::authority = tresury_account,
    payer = signer,
    seeds = [b"vesting_tresury", company_name.as_bytes()],
    bump
  )]
  pub tresury_account: Account<'info, TokenAccount>,

  pub system_program: Program<'info, System>,
  pub token_program: Interface<'info, TokenInterface>

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

#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount{
  pub beneficiary: Pubkey,
  //time stored in unixtime
  pub start_time: i64,
  pub end_time: i64,
  pub cliff_time: i64,
  pub vesting_account: Pubkey,
  pub total_amount: u64,
  pub total_withdrawn: u64,
  pub bump: u8

}

#[error_code]
pub enum ErrorCode{
  #[msg("Claiming is not available yet")]
  ClaimNotAvailbleYet,
  #[msg("Nothing is there to claim")]
  NothingToClaim
}


