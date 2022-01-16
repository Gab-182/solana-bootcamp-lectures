use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::EchoError;

struct Context<'a, 'b: 'a> {
    echo_buffer: &'a AccountInfo<'b>,
}

impl<'a, 'b: 'a> Context<'a, 'b> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let ctx = Self {
            echo_buffer: next_account_info(accounts_iter)?,
        };

        if !ctx.echo_buffer.is_writable {
            msg!("Echo Buffer account must be writable");
            return Err(EchoError::AccountMustBeWritable.into());
        }

        Ok(ctx)
    }
}

pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], data: Vec<u8>) -> ProgramResult {
    let ctx = Context::parse(accounts)?;

    let buffer = &mut (*ctx.echo_buffer.data).borrow_mut();

    if buffer.len() == 0 {
        msg!("Account has data length of 0, failing instruction");
        return Err(EchoError::AccountNotInitialized.into());
    }

    // if there is any non-zero data in the account, fail the instruction
    for index in 0..buffer.len() {
        if buffer[index] != 0 {
            msg!("Account has non-zero data at index {}", index);
            return Err(EchoError::AccountHasNonZeroData.into());
        }
    }

    let bytes_to_copy = buffer.len().min(data.len());
    msg!("Echo buffer account length: {}", buffer.len());
    msg!("Input data length: {}", data.len());

    // copy the bytes from the input data vector to the account
    for index in 0..bytes_to_copy {
        buffer[index] = data[index];
    }

    msg!("{:?}", *ctx.echo_buffer.data);

    msg!(
        "Successfully wrote {} bytes to account of size {}",
        bytes_to_copy,
        buffer.len()
    );

    Ok(())
}

// test cases:
// 1. successfully write and confirm when data size < buffer size
// 2. successfully write and confirm when data size > buffer size
// 3. successfully write and confirm when data size == buffer size
// 4. fail if account not allocated
// 5. fail when attempting to write an account that already has data
