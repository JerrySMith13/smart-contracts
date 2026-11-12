# smart-contracts

A small sandbox for learning how Ethereum and Solana keep state. It is not a
protocol and it is not an audit lab. Two counters, three languages, tests that
run on your machine.

## Ethereum: accounts, transactions, storage

An **account** is an address with a nonce, an ETH balance, and optionally
code. An externally owned account has no code; a person signs for it. A
contract account has code. That code is what runs when someone calls it.

A **transaction** is a signed message from an account. It can transfer ETH
and it can call a contract. If the call succeeds, the contract's storage
updates are kept. If it reverts, they are discarded. You pay gas either way.

**Contract storage** belongs to the contract account, not to the caller. It
is a key-value map of 32-byte slots. The compiler lays variables out in those
slots. In both counters here, `count` is slot 0. Only a transaction that
executes this contract can change it. Reading `count()` is a call against the
current state; it does not create a transaction and does not write storage.

The same idea is written twice:

- [`ethereum/src/Counter.sol`](ethereum/src/Counter.sol) — Solidity
- [`ethereum/src/Counter.vy`](ethereum/src/Counter.vy) — Vyper

[`ethereum/test/Counter.t.sol`](ethereum/test/Counter.t.sol) deploys each one
and checks the start value, `increment`, and `set`.

## Solidity and Vyper

Both compile to EVM bytecode. They share accounts, transactions, and storage.
A Solidity contract can call a Vyper contract, and the chain cannot tell the
source language from the account alone.

Solidity is the language Foundry is built around. It has inheritance,
modifiers, function overloading, and inline assembly. Vyper is smaller on
purpose. It looks like Python, and it leaves those features out so there is
less to misread. These two files only need one storage variable and two
writes, so they look almost the same.

The compilers are pinned: Solidity **0.8.38** (`pragma solidity 0.8.38`;
Forge downloads that compiler) and Vyper **0.4.3**
([`ethereum/requirements.txt`](ethereum/requirements.txt)). Vyper 0.4.3
targets the Prague EVM, and the Foundry profile does too. The Vyper contract
is deployed in the tests with `deployCode`, not a Solidity `import`: Foundry
1.9 reads a Vyper version pragma as a solc constraint when a Solidity file
imports the `.vy` source.

## Solana: the account is passed in

A Solana **program** is code stored in an executable account. Calling it does
not give it a private storage map. Every account the program may read or
write is listed on the instruction, and the program receives those accounts
as arguments.

The counter in [`solana/src/lib.rs`](solana/src/lib.rs) is 9 bytes inside an
account the caller supplies. The program writes it only when that account is
owned by the program and marked writable. Initialize, increment, and a
rejected foreign account are covered in
[`solana/tests/counter.rs`](solana/tests/counter.rs).

The tests use [Mollusk](https://docs.rs/mollusk-svm). Mollusk executes the
compiled program in-process. It does not start a validator. The program is
built with Pinocchio 0.12.0; Mollusk is 0.17.1. Anchor is not used.

## Run the Ethereum tests

Install Foundry (https://getfoundry.sh), then Vyper 0.4.3, then Forge. Forge
downloads Solidity 0.8.38 itself. This repo was checked with Foundry 1.9.0.

```sh
python3 -m pip install -r ethereum/requirements.txt
cd ethereum
forge test
```

`vyper` has to be on `PATH`. Foundry has no version field for it; the
requirements file is the pin. `forge test` compiles `Counter.sol` and
`Counter.vy` and runs both suites.

## Run the Solana tests

Install stable Rust and the Solana CLI (https://docs.anza.xyz). The CLI
provides `cargo build-sbf`. This repo was checked with Rust 1.99.1 and
Solana CLI 4.3.2 (platform-tools v1.57).

```sh
cd solana
cargo build-sbf
cargo test
```

`cargo build-sbf` writes `target/deploy/counter.so`. `cargo test` loads that
file. [`solana/.cargo/config.toml`](solana/.cargo/config.toml) points Mollusk
at `target/deploy`, so the test command does not need an extra variable.

## Continuous integration

[`.github/workflows/test.yml`](.github/workflows/test.yml) installs Foundry,
Vyper 0.4.3, the Solana CLI, and stable Rust, then runs `forge test` and
`cargo build-sbf` followed by `cargo test`.
