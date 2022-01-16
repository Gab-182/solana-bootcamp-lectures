use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::Sysvar,
};
use spl_token::state::Mint;

use crate::{
    error::EchoError,
    state::{VendingMachineBufferHeader, VENDING_MACHINE_BUFF_HEADER_SIZE},
};

use borsh::BorshSerialize;

struct Context<'a, 'b: 'a> {
    vending_machine_buffer: &'a AccountInfo<'b>,
    vending_machine_mint: &'a AccountInfo<'b>,
    payer: &'a AccountInfo<'b>,
    system_program: &'a AccountInfo<'b>,
}

impl<'a, 'b: 'a> Context<'a, 'b> {
    pub fn parse(accounts: &'a [AccountInfo<'b>]) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();

        let ctx = Self {
            vending_machine_buffer: next_account_info(accounts_iter)?,
            vending_machine_mint: next_account_info(accounts_iter)?,
            payer: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
        };

        if !ctx.vending_machine_buffer.is_writable {
            msg!("Authorized Echo Buffer account must be writable");
            return Err(EchoError::AccountMustBeWritable.into());
        }

        if !ctx.payer.is_signer {
            msg!("Payer must be signer");
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
    price: u64,
    buffer_size: usize,
) -> ProgramResult {
    let ctx = Context::parse(accounts)?;

    // need at least enough for the buffer header
    if buffer_size <= VENDING_MACHINE_BUFF_HEADER_SIZE {
        msg!(
            "Invalid buffer length {}, must be greater than header size {}",
            buffer_size,
            VENDING_MACHINE_BUFF_HEADER_SIZE
        );
        return Err(EchoError::InvalidInstructionInput.into());
    }

    let _mint = Mint::unpack_unchecked(&ctx.vending_machine_mint.data.borrow()).map_err(|e| {
        msg!("Invalid mint account");
        return e;
    })?;
    // verify that the PDA account is the correct address
    let (pda, bump_seed) = Pubkey::find_program_address(
        &[
            b"vending_machine",
            ctx.vending_machine_mint.key.as_ref(),
            &price.to_le_bytes(),
        ],
        program_id,
    );

    if *ctx.vending_machine_buffer.key != pda {
        msg!("Invalid authorized buffer address");
        return Err(EchoError::InvalidAccountAddress.into());
    }

    // call the system program to create the account
    let create_account_ix = create_account(
        &ctx.payer.key,
        &ctx.vending_machine_buffer.key,
        Rent::get()?.minimum_balance(buffer_size),
        buffer_size as u64,
        program_id,
    );

    invoke_signed(
        &create_account_ix,
        &[
            ctx.vending_machine_buffer.clone(),
            ctx.payer.clone(),
            ctx.system_program.clone(),
        ],
        &[&[
            b"vending_machine",
            ctx.vending_machine_mint.key.as_ref(),
            &price.to_le_bytes(),
            &[bump_seed],
        ]],
    )?;

    // the full data buffer
    let buffer = &mut (*ctx.vending_machine_buffer.data).borrow_mut();

    // slice of the buffer used for the header
    let buffer_header = VendingMachineBufferHeader { bump_seed, price };

    buffer[0..VENDING_MACHINE_BUFF_HEADER_SIZE]
        .copy_from_slice(&buffer_header.try_to_vec().unwrap());

    msg!("Vending machine buffer len: {}", buffer_size);
    msg!("Bump seed: {}", bump_seed);
    msg!("Price: {}", price);

    Ok(())
}
