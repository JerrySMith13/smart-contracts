//! Counter program.
//!
//! A Solana program has no storage of its own. The counter lives in an account
//! the caller passes in. This program only writes that account when it owns
//! the account and the instruction marked the account writable.
//!
//! The caller creates the account first (system program `create_account`, with
//! this program as the owner and [`ACCOUNT_LEN`] bytes of data). Creation is
//! not done here. The tests hand the program an account that is already the
//! right size and owner.
//!
//! Account bytes, little-endian:
//!
//! ```text
//! [0]     tag    0 = empty, 1 = initialized
//! [1..9]  count  u64
//! ```
//!
//! Instruction data is one byte: [`IX_INITIALIZE`] or [`IX_INCREMENT`].

#![no_std]

use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

/// Create the counter in an empty account. Writes tag = 1 and count = 0.
pub const IX_INITIALIZE: u8 = 0;
/// Add one to an initialized counter.
pub const IX_INCREMENT: u8 = 1;

/// `tag` plus a `u64`.
pub const ACCOUNT_LEN: usize = 9;

const TAG_EMPTY: u8 = 0;
const TAG_INITIALIZED: u8 = 1;

// The loader calls `entrypoint`. This crate is `no_std`, so it brings its own
// panic handler. `entrypoint!` would install the std panic hook instead, and
// the sBPF target has no std. Host tests never execute this: they load the
// compiled `.so`.
#[cfg(target_os = "solana")]
pinocchio::program_entrypoint!(process_instruction);
#[cfg(target_os = "solana")]
pinocchio::nostd_panic_handler!();
// The program does not allocate. This allocator refuses any attempt to.
#[cfg(target_os = "solana")]
pinocchio::no_allocator!();

/// `program_id` is this program's address. `accounts` are exactly the accounts
/// the instruction listed, in that order. `instruction_data` is the opaque
/// byte string the instruction carried.
pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let tag = instruction_data
        .first()
        .copied()
        .ok_or(ProgramError::InvalidInstructionData)?;

    // The counter is account 0. Nothing else is required.
    let counter = accounts
        .first_mut()
        .ok_or(ProgramError::NotEnoughAccountKeys)?;

    // Ownership is the program's check. The runtime does not do it for us.
    // An account owned by someone else is not this program's storage.
    if !counter.owned_by(program_id) {
        return Err(ProgramError::IllegalOwner);
    }
    // The runtime rejects writes to an account the instruction did not mark
    // writable. Checking here makes that failure a program error.
    if !counter.is_writable() {
        return Err(ProgramError::Immutable);
    }

    let mut data = counter.try_borrow_mut()?;
    if data.len() != ACCOUNT_LEN {
        return Err(ProgramError::InvalidAccountData);
    }

    match tag {
        IX_INITIALIZE => initialize(&mut data),
        IX_INCREMENT => increment(&mut data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

fn initialize(data: &mut [u8]) -> ProgramResult {
    if data[0] != TAG_EMPTY {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    data[0] = TAG_INITIALIZED;
    data[1..].copy_from_slice(&0u64.to_le_bytes());
    Ok(())
}

fn increment(data: &mut [u8]) -> ProgramResult {
    if data[0] != TAG_INITIALIZED {
        return Err(ProgramError::UninitializedAccount);
    }
    let current_bytes: [u8; 8] = data[1..]
        .try_into()
        .map_err(|_| ProgramError::InvalidAccountData)?;
    let next = u64::from_le_bytes(current_bytes)
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    data[1..].copy_from_slice(&next.to_le_bytes());
    Ok(())
}
