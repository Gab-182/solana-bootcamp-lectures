use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use borsh::BorshDeserialize;

use crate::{
    error::EchoError,
    state::{AuthorizedBufferHeader, AUTH_BUFF_HEADER_SIZE},
};

struct Context<'a, 'b: 'a> {
    authorized_buffer: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
}

impl<'a, 'b: 'a> Context<'a, 'b> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let ctx = Self {
            authorized_buffer: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
        };

        if !ctx.authorized_buffer.is_writable {
            msg!("Authorized Echo Buffer account must be writable");
            return Err(EchoError::AccountMustBeWritable.into());
        }

        if !ctx.authority.is_signer {
            msg!("Authority account must be signer");
            return Err(EchoError::MissingRequiredSignature.into());
        }

        Ok(ctx)
    }
}

pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], data: Vec<u8>) -> ProgramResult {
    let ctx = Context::parse(accounts)?;

    let buffer = &mut (*ctx.authorized_buffer.data).borrow_mut();

    // check the size of the account before trying to read it
    if buffer.len() < AUTH_BUFF_HEADER_SIZE {
        msg!("Invalid authorized buffer size, {}", buffer.len());
        return Err(EchoError::AccountNotInitialized.into());
    }

    // in order to validate the PDA address, we first read it to access the buffer seed
    let buffer_header = AuthorizedBufferHeader::try_from_slice(&buffer[..AUTH_BUFF_HEADER_SIZE])?;

    // verify that the PDA account is the correct address
    let pda = Pubkey::create_program_address(
        &[
            b"authority",
            ctx.authority.key.as_ref(),
            &buffer_header.buffer_seed.to_le_bytes(),
            &[buffer_header.bump_seed],
        ],
        program_id,
    )?;

    if pda != *ctx.authorized_buffer.key {
        msg!("Invalid account address or authority");
        return Err(EchoError::InvalidAccountAddress.into());
    }

    // this is the 'rest' of the account's data (beyond the header info)
    let buffer_data = &mut buffer[AUTH_BUFF_HEADER_SIZE..];

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
