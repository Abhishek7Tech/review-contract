use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Sealed},
};
use thiserror::Error;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct ReviewState {
    pub is_initialized: bool,
    pub title: String,
    pub description: String,
    pub rating: u32,
}

impl IsInitialized for ReviewState {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Sealed for ReviewState {}

#[derive(Debug, Error)]
pub enum ReviewError {
    #[error("Account not initialized yet.")]
    UninitialzedAccount,

    #[error("Rating should be between 1 to 10.")]
    InvalidRating,

    #[error("PDA mismatched.")]
    InvalidPDA,
}

impl From<ReviewError> for ProgramError {
    fn from(e: ReviewError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
