//! Mollusk runs the compiled program inside this process.
//! There is no local validator and no network.
//!
//! Build first so `target/deploy/counter.so` exists:
//!
//! ```text
//! cargo build-sbf
//! cargo test
//! ```
//!
//! `.cargo/config.toml` sets `SBF_OUT_DIR` so Mollusk finds that file.

use counter::{ACCOUNT_LEN, IX_INCREMENT, IX_INITIALIZE};
use mollusk_svm::{result::ProgramResult as ExecResult, Mollusk};
use solana_account::Account;
use solana_instruction::{AccountMeta, Instruction};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

/// Plenty of lamports for a 9-byte account. Mollusk checks rent exemption.
const LAMPORTS: u64 = 1_000_000_000;

fn program_id() -> Pubkey {
    Pubkey::from([1u8; 32])
}

fn counter_key() -> Pubkey {
    Pubkey::from([2u8; 32])
}

fn harness() -> Mollusk {
    // Loads `counter.so` from SBF_OUT_DIR (target/deploy).
    Mollusk::new(&program_id(), "counter")
}

/// An account the program may treat as its counter: right length, zeroed, owned
/// by `owner`. A real client would create this with the system program.
fn counter_account(owner: Pubkey) -> Account {
    Account::new_with_data(LAMPORTS, vec![0u8; ACCOUNT_LEN], &owner)
}

fn instruction(program: Pubkey, counter: Pubkey, opcode: u8) -> Instruction {
    Instruction {
        program_id: program,
        accounts: vec![AccountMeta::new(counter, false)],
        data: vec![opcode],
    }
}

fn count_of(account: &Account) -> u64 {
    u64::from_le_bytes(account.data[1..9].try_into().unwrap())
}

#[test]
fn initialize_writes_zero() {
    let program = program_id();
    let key = counter_key();
    let mollusk = harness();

    let result = mollusk.process_instruction(
        &instruction(program, key, IX_INITIALIZE),
        &[(key, counter_account(program))],
    );

    assert_eq!(result.program_result, ExecResult::Success);
    let stored = result.get_account(&key).expect("counter account");
    assert_eq!(stored.data[0], 1, "tag marks the account initialized");
    assert_eq!(count_of(stored), 0);
}

#[test]
fn increment_adds_one() {
    let program = program_id();
    let key = counter_key();
    let mollusk = harness();

    let initialized = mollusk.process_instruction(
        &instruction(program, key, IX_INITIALIZE),
        &[(key, counter_account(program))],
    );
    assert_eq!(initialized.program_result, ExecResult::Success);
    let stored = initialized
        .get_account(&key)
        .expect("counter account")
        .clone();

    let once = mollusk.process_instruction(
        &instruction(program, key, IX_INCREMENT),
        &[(key, stored.clone())],
    );
    assert_eq!(once.program_result, ExecResult::Success);
    let stored = once.get_account(&key).expect("counter account").clone();
    assert_eq!(count_of(&stored), 1);

    let twice =
        mollusk.process_instruction(&instruction(program, key, IX_INCREMENT), &[(key, stored)]);
    assert_eq!(twice.program_result, ExecResult::Success);
    assert_eq!(count_of(twice.get_account(&key).unwrap()), 2);
}

#[test]
fn rejects_account_owned_by_someone_else() {
    let program = program_id();
    let key = counter_key();
    // Same size, same lamports, but a different owner. The program must refuse
    // to use it as its counter.
    let stranger = Pubkey::from([9u8; 32]);
    let mollusk = harness();

    let result = mollusk.process_instruction(
        &instruction(program, key, IX_INITIALIZE),
        &[(key, counter_account(stranger))],
    );

    assert_eq!(
        result.program_result,
        ExecResult::Failure(ProgramError::IllegalOwner)
    );
}
