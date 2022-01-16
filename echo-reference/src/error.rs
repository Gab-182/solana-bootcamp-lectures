use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug, Copy, Clone, FromPrimitive, PartialEq)]
pub enum EchoError {
    #[error("Account must be writable.")]
    AccountMustBeWritable,
    #[error("Account not initialized.")]
    AccountNotInitialized,
    #[error("Account contains non-zero data.")]
    AccountHasNonZeroData,
    #[error("Missing required signature.")]
    MissingRequiredSignature,
    #[error("Invalid program address.")]
    InvalidProgramAddress,
    #[error("Invalid account address.")]
    InvalidAccountAddress,
    #[error("Invalid instruction input.")]
    InvalidInstructionInput,
    #[error("Invalid account data.")]
    InvalidAccountData,
    #[error("Default error.")]
    DefaultError,
    #[error("Instruction not implemented.")]
    NotImplemented,
    #[error("Insufficient funds.")]
    InsufficientFunds,
}

impl From<EchoError> for ProgramError {
    fn from(e: EchoError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
