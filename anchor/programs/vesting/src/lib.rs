#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked}};

declare_id!("AsjZ3kWAUSQRNt2pZVeJkywhZ6gpLpHZmJjduPmKZDZZ");

//creating a smartcontract program
#[program]
pub mod vesting {
  use anchor_spl::token_interface;

use super::*;

  pub fn create_vesting(ctx: Context<CreateVestingAccount>, company_name: String) -> Result<()>{
    //feeding info in the account
    *ctx.accounts.vesting_account = VestingAccount {
      owner: ctx.accounts.signer.key(),
      mint: ctx.accounts.mint.key(),
      treasury_token_account: ctx.accounts.treasury_token_account.key(),
      company_name: company_name,
      treasury_bump: ctx.bumps.treasury_token_account,
      bump: ctx.bumps.vesting_account
    };

    Ok(())
  }

  pub fn create_employee(ctx: Context<CreateEmployee>, start_time: i64, end_time: i64, cliff_time: i64, total_amount: u64) -> Result<()> {
    *ctx.accounts.employee_account = EmployeeAccount {
      beneficiary: ctx.accounts.beneficiary.key(),
      vesting_account: ctx.accounts.vesting_account.key(),
      start_time,
      end_time,
      cliff_time,
      total_amount,
      total_withdrawn: 0,
      bump: ctx.bumps.employee_account
    };
    Ok(())
  }

  pub fn claim_tokens(ctx: Context<ClaimTokens>, _company_name: String) -> Result<()> {

    let employee_account = &mut ctx.accounts.employee_account;

    let curr_time = Clock::get()?.unix_timestamp;

    if curr_time < employee_account.cliff_time {
      // !into means here ??
      return Err(ErrorCode::ClaimNotAvailableYet.into());
    }

    let time_since_start = curr_time.saturating_sub(employee_account.start_time);

    let total_vesting_time= employee_account.end_time.saturating_sub(employee_account.start_time);

    if total_vesting_time == 0 {
      return Err(ErrorCode::InvalidVestingPeriod.into())
    }
    
    let vested_amount = if curr_time >= employee_account.end_time {
      employee_account.total_amount
    } else {
      match employee_account.total_amount.checked_mul(time_since_start as u64){
        Some(product) => product/total_vesting_time as u64,
        None => {
          return Err(ErrorCode::CalculationOverflow.into())
        }
      }
    };

    let claimable_amount = vested_amount.saturating_sub(employee_account.total_withdrawn);

    if claimable_amount == 0 {
      return Err(ErrorCode::NothingToClaim.into());
    }

    // this sets up imp info 

    let transfer_api_accounts = TransferChecked {
      from: ctx.accounts.treasury_token_account.to_account_info(),
      mint: ctx.accounts.mint.to_account_info(),
      // ! verify it with employee_account
      to: ctx.accounts.employee_token_account.to_account_info(),
      authority: ctx.accounts.treasury_token_account.to_account_info()
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    //we require signer seeds 

    let signer_seeds: &[&[&[u8]]] = &[
      &[
        b"vesting_treasury",
        ctx.accounts.vesting_account.company_name.as_ref(),
        &[ctx.accounts.vesting_account.treasury_bump],
      ]
    ];
    let cpi_context = CpiContext::new(cpi_program, transfer_api_accounts).with_signer(signer_seeds);

    let decimals = ctx.accounts.mint.decimals;
    token_interface::transfer_checked(cpi_context, claimable_amount as u64,decimals)?;
    employee_account.total_withdrawn += claimable_amount;
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
    seeds = [b"employee_account", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
    // this will verify the both the fields in the emploeyee account matched with the provided fields account pub key
    has_one = beneficiary,
    has_one = vesting_account,
    // bump seeds used to derive the pda so here it will verify that pda derivation is correct
    bump = employee_account.bump,
  )]
  pub employee_account: Account<'info, EmployeeAccount>,

  #[account(
    mut,
    seeds = [company_name.as_ref()],
    bump = vesting_account.bump,
    has_one = treasury_token_account, 
    has_one = mint
  )]
  pub vesting_account: Account<'info, VestingAccount>,

  pub mint: InterfaceAccount<'info, Mint>,
  #[account(mut)]
  pub treasury_token_account: InterfaceAccount<'info, TokenAccount>,

  //creating ATA to store those spl tokens
  // this will need mint, authority
  #[account(
    init_if_needed,
    payer = beneficiary,
    associated_token::mint = mint,
    associated_token::authority= beneficiary,
    associated_token::token_program = token_program
  )]
  pub employee_token_account: InterfaceAccount<'info, TokenAccount>,

  pub token_program: Interface<'info, TokenInterface>,
  pub associated_token_program: Program<'info, AssociatedToken>,
  pub system_program: Program<'info, System>
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
  pub mint: InterfaceAccount<'info, Mint>,

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


  pub token_program: Interface<'info, TokenInterface>,
  pub system_program: Program<'info,System>

}

#[derive(Accounts)]
pub struct CreateEmployee<'info>{
  #[account(mut)]
  pub owner: Signer<'info>,

  // !what is system account
  pub beneficiary: SystemAccount<'info>,

  //making sure that the owner of the vesting account is the signer of this instruction
  // !here how we are getting the vesting account without usoing the pda
  #[account(
    has_one = owner,
  )]
  pub vesting_account: Account<'info, VestingAccount>,

  #[account(
    init,
    payer = owner,
    seeds = [b"employee_account", beneficiary.key().as_ref(), vesting_account.key().as_ref()],
    space = 8 + EmployeeAccount::INIT_SPACE,
    bump

  )]
  pub employee_account: Account<'info, EmployeeAccount>,

  pub system_program: Program<'info, System>



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
#[account]
#[derive(InitSpace)]
pub struct EmployeeAccount {
  pub beneficiary: Pubkey,
  pub start_time: i64,
  pub end_time: i64,
  pub cliff_time: i64,
  //as this will linked to a vesting account to claim back those tokens
  pub vesting_account: Pubkey,
  pub total_amount: u64,
  pub total_withdrawn: u64,
  pub bump: u8
}

#[error_code]
pub enum ErrorCode{
  #[msg("Claiming is not avaiable yet.")]
  ClaimNotAvailableYet,
  #[msg("Invalid Vesting Period.")]
  InvalidVestingPeriod,
  #[msg("Calculation Overflow")]
  CalculationOverflow,
  #[msg("Nothing To Claim")]
  NothingToClaim
}
