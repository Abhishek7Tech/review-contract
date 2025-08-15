use borsh::BorshSerialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo}, borsh1::try_from_slice_unchecked, entrypoint::ProgramResult, msg, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey, system_instruction, system_program, sysvar::{rent::Rent, Sysvar}
};
use solana_program::program_pack::IsInitialized;
use crate::{instructions::ReviewInstructions, state::ReviewError, state::ReviewState};

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = ReviewInstructions::unpack(instruction_data)?;

    match instruction {
        ReviewInstructions::AddReview {
            title,
            description,
            rating,
        } => add_review(program_id, accounts, title, description, rating)?,
        ReviewInstructions::UpdateReview {
            title,
            description,
            rating,
        } => update_review(program_id, accounts, title, description, rating)?,
        _ => return Err(ProgramError::InvalidInstructionData),
    };

    Ok(())
}

pub fn add_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    description: String,
    rating: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let initializer = next_account_info(accounts_iter)?;
    msg!("Initializer {}", initializer.key);
    let pda_account = next_account_info(accounts_iter)?;
    msg!("PDA Account {:?}", pda_account);
    let system_program = next_account_info(accounts_iter)?;

    if rating > 10 || rating < 1 {
        return Err(ReviewError::InvalidRating.into());
    }

    if !initializer.is_signer {
        return Err(ReviewError::UninitialzedAccount.into());
    }

    let account_len = 1000 as usize;
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(account_len);

    let (pda, bump_seed) = Pubkey::find_program_address(
        &[initializer.key.as_ref(), title.as_bytes().as_ref()],
        program_id,
    );
    msg!("PDA Bump {:?}", pda);

    if pda != *pda_account.key {
        return  Err(ReviewError::InvalidPDA.into());
    }
    invoke_signed(
        &system_instruction::create_account(
            initializer.key,
            pda_account.key,
            rent_lamports,
            account_len.try_into().unwrap(),
            program_id,
        ),
        &[
            initializer.clone(),
            pda_account.clone(),
            system_program.clone(),
        ],
        &[&[
            initializer.key.as_ref(),
            title.as_bytes().as_ref(),
            &[bump_seed],
        ]],
    )?;

    let mut account_data = try_from_slice_unchecked::<ReviewState>(&pda_account.data.borrow())?;

    if account_data.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    account_data.description = description;
    account_data.title = title;
    account_data.rating = rating;
    account_data.is_initialized = true;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn update_review(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    title: String,
    description: String,
    rating: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let initializer = next_account_info(accounts_iter)?;
    msg!("Initializer {}", initializer.key);
    let pda_account = next_account_info(accounts_iter)?;
    msg!("PDA Account {:?}", pda_account);

    if pda_account.owner != program_id {
        return  Err(ProgramError::IllegalOwner);
    }

    if !initializer.is_signer {
        return  Err(ProgramError::MissingRequiredSignature);
    }

    let (pda, _bump_seed) = Pubkey::find_program_address(&[initializer.key.as_ref(), title.as_bytes().as_ref()], program_id);
    msg!("PDA Bump {:?}", pda);
    
    if pda != *pda_account.key {
        return  Err(ReviewError::InvalidPDA.into());
    }

    let mut account_data = try_from_slice_unchecked::<ReviewState>(&pda_account.data.borrow())?;

    if !account_data.is_initialized() {
        return  Err(ReviewError::UninitialzedAccount.into());
    }

    if rating > 10 || rating < 1 {
        return  Err(ReviewError::InvalidRating.into());
    }

    account_data.rating = rating;
    account_data.description = description;

    account_data.serialize(&mut &mut pda_account.data.borrow_mut()[..])?;


    Ok(())
}
