use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::EchoInstruction;

pub mod authorized_echo;
pub mod echo;
pub mod initialize_authorized_echo;
pub mod initialize_vending_machine_echo;
pub mod vending_machine_echo;

pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EchoInstruction::try_from_slice(instruction_data)
            .map_err(|_| ProgramError::InvalidInstructionData)?;

        match instruction {
            EchoInstruction::Echo { data } => {
                msg!("Instruction: Echo");
                echo::process(program_id, accounts, data)?;
            }
            EchoInstruction::InitializeAuthorizedEcho {
                buffer_seed,
                buffer_size,
            } => {
                msg!("Instruction: InitializeAuthorizedEcho");
                initialize_authorized_echo::process(
                    program_id,
                    accounts,
                    buffer_seed,
                    buffer_size,
                )?;
            }
            EchoInstruction::AuthorizedEcho { data } => {
                msg!("Instruction: AuthorizedEcho");
                authorized_echo::process(program_id, accounts, data)?;
            }
            EchoInstruction::InitializeVendingMachineEcho { price, buffer_size } => {
                msg!("Instruction: InitializeVendingMachineEcho");
                initialize_vending_machine_echo::process(program_id, accounts, price, buffer_size)?;
            }
            EchoInstruction::VendingMachineEcho { data } => {
                msg!("Instruction: VendingMachineEcho");
                vending_machine_echo::process(program_id, accounts, data)?;
            }
        }

        Ok(())
    }
}
