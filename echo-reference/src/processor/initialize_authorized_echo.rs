use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::Sysvar,
};

use crate::{
    error::EchoError,
    state::{AuthorizedBufferHeader, AUTH_BUFF_HEADER_SIZE},
};

use borsh::BorshSerialize;

struct Context<'a, 'b: 'a> {
    authorized_buffer: &'a AccountInfo<'b>,
    authority: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
}

impl<'a, 'b: 'a> Context<'a, 'b> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let ctx = Self {
            authorized_buffer: next_account_info(accounts_iter)?,
            authority: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        if !ctx.authorized_buffer.is_writable {
            msg!("Authorized Echo Buffer account must be writable");
            return Err(EchoError::AccountMustBeWritable.into());
        }

        if !ctx.authority.is_signer {
            msg!("Authority account must be signer");
            return Err(EchoError::MissingRequiredSignature.into());
        }

        if *ctx.system_program.key != SYSTEM_PROGRAM_ID {
            msg!("Invalid system program");
            return Err(EchoError::InvalidProgramAddress.into());
        }

        Ok(ctx)
    }
}

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    buffer_seed: u64,
    buffer_size: usize,
) -> ProgramResult {
    let ctx = Context::parse(accounts)?;

    // need at least enough for the buffer header
    if buffer_size <= AUTH_BUFF_HEADER_SIZE {
        msg!(
            "Invalid buffer length {}, must be greater than header size {}",
            buffer_size,
            AUTH_BUFF_HEADER_SIZE
        );
        return Err(EchoError::InvalidInstructionInput.into());
    }

    // verify that the PDA account is the correct address
    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            b"authority",
            ctx.authority.key.as_ref(),
            &buffer_seed.to_le_bytes(),
        ],
        program_id,
    );

    if *ctx.authorized_buffer.key != pda {
        msg!("Invalid authorized buffer address");
        return Err(EchoError::InvalidAccountAddress.into());
    }

    // call the system program to create the account
    let create_account_ix = create_account(
        &ctx.authority.key,
        &ctx.authorized_buffer.key,
        Rent::get()?.minimum_balance(buffer_size),
        buffer_size as u64,
        program_id,
    );

    invoke_signed(
        &create_account_ix,
        &[
            ctx.authorized_buffer.clone(),
            ctx.authority.clone(),
            ctx.system_program.clone(),
        ],
        &[&[
            b"authority",
            ctx.authority.key.as_ref(),
            &buffer_seed.to_le_bytes(),
            &[bump_seed],
        ]],
    )?;

    // the full data buffer
    let buffer = &mut (*ctx.authorized_buffer.data).borrow_mut();

    // slice of the buffer used for the header
    let buffer_header = AuthorizedBufferHeader {
        bump_seed,
        buffer_seed,
    };

    buffer[0..AUTH_BUFF_HEADER_SIZE].copy_from_slice(&buffer_header.try_to_vec().unwrap());

    msg!("Authorized buffer len: {}", buffer_size);
    msg!("Bump seed: {}", bump_seed);
    msg!("Buffer seed: {}", buffer_seed);

    Ok(())
}
