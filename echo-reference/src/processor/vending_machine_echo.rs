use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::{Account as TokenAccount, Mint};

use borsh::BorshDeserialize;

use crate::{
    error::EchoError,
    state::{VendingMachineBufferHeader, VENDING_MACHINE_BUFF_HEADER_SIZE},
};

struct Context<'a, 'b: 'a> {
    vending_machine_buffer: &'a AccountInfo<'b>,
    user: &'a AccountInfo<'b>,
    user_token_account: &'a AccountInfo<'b>,
    vending_machine_mint: &'a AccountInfo<'b>,
    token_program: &'a AccountInfo<'b>,
}

impl<'a, 'b: 'a> Context<'a, 'b> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let ctx = Self {
            vending_machine_buffer: next_account_info(accounts_iter)?,
            user: next_account_info(accounts_iter)?,
            user_token_account: next_account_info(accounts_iter)?,
            vending_machine_mint: next_account_info(accounts_iter)?,
            token_program: next_account_info(accounts_iter)?,
        };

        if !ctx.vending_machine_buffer.is_writable {
            msg!("Authorized Echo Buffer account must be writable");
            return Err(EchoError::AccountMustBeWritable.into());
        }

        if !ctx.user_token_account.is_writable {
            msg!("Authorized Echo Buffer account must be writable");
            return Err(EchoError::AccountMustBeWritable.into());
        }

        if !ctx.user.is_signer {
            msg!("User account must be signer");
            return Err(EchoError::MissingRequiredSignature.into());
        }

        Ok(ctx)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: Vec<u8>) -> ProgramResult {
    let ctx = Context::parse(accounts)?;

    let _mint = Mint::unpack_unchecked(&ctx.vending_machine_mint.data.borrow()).map_err(|e| {
        msg!("Invalid mint account");
        return e;
    })?;
    let user_token_account = TokenAccount::unpack_unchecked(&ctx.user_token_account.data.borrow())
        .map_err(|e| {
            msg!("Invalid token account");
            return e;
        })?;

    if user_token_account.owner != *ctx.user.key {
        msg!("Invalid token account owner");
        return Err(EchoError::InvalidAccountData.into());
    }

    if user_token_account.mint != *ctx.vending_machine_mint.key {
        msg!("Invalid token account mint");
        return Err(EchoError::InvalidAccountData.into());
    }

    let buffer = &mut (*ctx.vending_machine_buffer.data).borrow_mut();

    // check the size of the account before trying to read it
    if buffer.len() < VENDING_MACHINE_BUFF_HEADER_SIZE {
        msg!("Invalid vending machine buffer size, {}", buffer.len());
        return Err(EchoError::AccountNotInitialized.into());
    }

    // in order to validate the PDA address, we first read it to access the buffer seed
    let buffer_header =
        VendingMachineBufferHeader::try_from_slice(&buffer[..VENDING_MACHINE_BUFF_HEADER_SIZE])?;

    if user_token_account.amount < buffer_header.price {
        msg!("Token account has insufficient funds");
        return Err(EchoError::InsufficientFunds.into());
    }

    // verify that the PDA account is the correct address
    let pda = Pubkey::create_program_address(
        &[
            b"vending_machine",
            ctx.vending_machine_mint.key.as_ref(),
            &buffer_header.price.to_le_bytes(),
            &[buffer_header.bump_seed],
        ],
        program_id,
    )?;

    if pda != *ctx.vending_machine_buffer.key {
        msg!("Invalid account address or authority");
        return Err(EchoError::InvalidAccountAddress.into());
    }

    // Burn the vending machine tokens to authorize the echo
    invoke(
        &spl_token::instruction::burn(
            ctx.token_program.key,
            ctx.user_token_account.key,
            ctx.vending_machine_mint.key,
            ctx.user.key,
            &[],
            buffer_header.price,
        )?,
        &[
            ctx.token_program.clone(),
            ctx.user_token_account.clone(),
            ctx.vending_machine_mint.clone(),
            ctx.user.clone(),
        ],
    )?;

    // this is the 'rest' of the account's data (beyond the header info)
    let buffer_data = &mut buffer[VENDING_MACHINE_BUFF_HEADER_SIZE..];

    // loop over each byte in the rest of account's data
    for index in 0..buffer_data.len() {
        buffer_data[index] = match index < data.len() {
            true => data[index],
            false => 0,
        };
    }

    Ok(())
}

// test cases:
