// SPDX-License-Identifier: AGPL-3.0-only
//! # USP-01 Token Example Contract
//!
//! Minimal USP-01 compliant fungible token using the LOS SDK.
//! This is a simplified example for learning — the full production
//! contract is in `crates/los-contracts/src/usp01_token.rs`.
//!
//! ## Features
//! - Fixed supply at deployment (assigned to creator)
//! - Transfer tokens between accounts
//! - Approve + TransferFrom (allowance mechanism)
//! - Burn (permanent supply reduction)
//! - Balance / Allowance / TotalSupply / TokenInfo queries
//! - All amounts in atomic units (`u128`) — NO floating-point
//!
//! ## State Layout
//! - `usp01:init`              → "1" when initialized
//! - `usp01:name`              → Token name
//! - `usp01:symbol`            → Ticker symbol (max 8 chars)
//! - `usp01:decimals`          → Decimal places (0-18)
//! - `usp01:total_supply`      → Total supply (decimal string)
//! - `usp01:owner`             → Creator address
//! - `bal:{address}`           → Balance (decimal string)
//! - `allow:{owner}:{spender}` → Allowance (decimal string)
//!
//! ## Compilation
//! ```bash
//! cargo build --target wasm32-unknown-unknown --release \
//!     -p los-contract-examples --bin token --features sdk
//! ```

#![no_std]
#![no_main]

extern crate alloc;
extern crate los_sdk;

use alloc::format;
use alloc::string::String;
use los_sdk::*;

// ─────────────────────────────────────────────────────────────
// HELPERS (no_std safe, u128 integer only)
// ─────────────────────────────────────────────────────────────

/// Parse a decimal string to u128. Returns 0 on failure.
fn parse_u128(s: &str) -> u128 {
    let mut result: u128 = 0;
    for b in s.as_bytes() {
        if *b < b'0' || *b > b'9' {
            return 0;
        }
        result = match result.checked_mul(10) {
            Some(v) => v,
            None => return 0,
        };
        result = match result.checked_add((*b - b'0') as u128) {
            Some(v) => v,
            None => return 0,
        };
    }
    result
}

/// Convert u128 to decimal string.
fn u128_to_str(val: u128) -> String {
    if val == 0 {
        return String::from("0");
    }
    let mut digits = alloc::vec::Vec::new();
    let mut v = val;
    while v > 0 {
        digits.push(b'0' + (v % 10) as u8);
        v /= 10;
    }
    digits.reverse();
    String::from_utf8(digits).unwrap_or_default()
}

fn bal_key(addr: &str) -> String {
    format!("bal:{}", addr)
}

fn allow_key(owner: &str, spender: &str) -> String {
    format!("allow:{}:{}", owner, spender)
}

fn get_balance(addr: &str) -> u128 {
    parse_u128(&state::get_str(&bal_key(addr)).unwrap_or_default())
}

fn set_balance(addr: &str, amount: u128) {
    state::set_str(&bal_key(addr), &u128_to_str(amount));
}

fn get_allowance(owner: &str, spender: &str) -> u128 {
    parse_u128(&state::get_str(&allow_key(owner, spender)).unwrap_or_default())
}

fn set_allowance(owner: &str, spender: &str, amount: u128) {
    state::set_str(&allow_key(owner, spender), &u128_to_str(amount));
}

fn get_total_supply() -> u128 {
    parse_u128(&state::get_str("usp01:total_supply").unwrap_or_default())
}

fn set_total_supply(val: u128) {
    state::set_str("usp01:total_supply", &u128_to_str(val));
}

fn is_initialized() -> bool {
    state::get_str("usp01:init").map_or(false, |v| v == "1")
}

fn fail(msg: &str) -> i32 {
    set_return_str(&format!(r#"{{"success":false,"msg":"{}"}}"#, msg));
    1
}

fn ok_data(data: &str) -> i32 {
    set_return_str(&format!(r#"{{"success":true,"data":{}}}"#, data));
    0
}

/// Escape a string for JSON (minimal).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(c),
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────
// INIT — Called once at deployment
// ─────────────────────────────────────────────────────────────

/// Initialize a new USP-01 token.
///
/// Args:
///   0: name (string, 1-64 chars)
///   1: symbol (string, 1-8 chars)
///   2: decimals (u8, 0-18)
///   3: total_supply (u128 decimal string)
#[no_mangle]
pub extern "C" fn init() -> i32 {
    if is_initialized() {
        return fail("Already initialized");
    }

    let name = match arg(0) {
        Some(n) if !n.is_empty() && n.len() <= 64 => n,
        _ => return fail("name required (1-64 chars)"),
    };
    let symbol = match arg(1) {
        Some(s) if !s.is_empty() && s.len() <= 8 => s,
        _ => return fail("symbol required (1-8 chars)"),
    };
    let decimals_str = arg(2).unwrap_or_default();
    let decimals = match decimals_str.parse::<u64>() {
        Ok(d) if d <= 18 => d,
        _ => return fail("decimals must be 0-18"),
    };
    let total_supply = parse_u128(&arg(3).unwrap_or_default());
    if total_supply == 0 {
        return fail("total_supply must be > 0");
    }

    let creator = caller();
    if creator.is_empty() {
        return fail("caller address not available");
    }

    // Store metadata (USP-01 standard keys)
    state::set_str("usp01:init", "1");
    state::set_str("usp01:name", &name);
    state::set_str("usp01:symbol", &symbol);
    state::set_str("usp01:decimals", &format!("{}", decimals));
    set_total_supply(total_supply);
    state::set_str("usp01:owner", &creator);

    // Assign entire supply to creator
    set_balance(&creator, total_supply);

    event::emit(
        "USP01:Init",
        &format!(
            r#"{{"name":"{}","symbol":"{}","decimals":{},"total_supply":"{}","creator":"{}"}}"#,
            json_escape(&name),
            json_escape(&symbol),
            decimals,
            u128_to_str(total_supply),
            json_escape(&creator)
        ),
    );

    set_return_str(&format!(
        r#"{{"success":true,"name":"{}","symbol":"{}","total_supply":"{}","owner":"{}"}}"#,
        json_escape(&name),
        json_escape(&symbol),
        u128_to_str(total_supply),
        json_escape(&creator)
    ));
    0
}

// ─────────────────────────────────────────────────────────────
// TRANSFER
// ─────────────────────────────────────────────────────────────

/// Transfer tokens from caller to recipient.
/// Args: 0: to, 1: amount
#[no_mangle]
pub extern "C" fn transfer() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    let to = match arg(0) {
        Some(a) if !a.is_empty() => a,
        _ => return fail("recipient address required"),
    };
    let amount = parse_u128(&arg(1).unwrap_or_default());
    if amount == 0 {
        return fail("amount must be > 0");
    }
    let from = caller();
    if from == to {
        return fail("cannot transfer to self");
    }

    let from_bal = get_balance(&from);
    if from_bal < amount {
        return fail("insufficient balance");
    }

    let new_from = match from_bal.checked_sub(amount) {
        Some(v) => v,
        None => return fail("arithmetic underflow"),
    };
    set_balance(&from, new_from);

    let to_bal = get_balance(&to);
    let new_to = match to_bal.checked_add(amount) {
        Some(v) => v,
        None => return fail("arithmetic overflow"),
    };
    set_balance(&to, new_to);

    event::emit(
        "USP01:Transfer",
        &format!(
            r#"{{"from":"{}","to":"{}","amount":"{}"}}"#,
            json_escape(&from),
            json_escape(&to),
            u128_to_str(amount)
        ),
    );

    set_return_str(&format!(
        r#"{{"success":true,"from":"{}","to":"{}","amount":"{}"}}"#,
        json_escape(&from),
        json_escape(&to),
        u128_to_str(amount)
    ));
    0
}

// ─────────────────────────────────────────────────────────────
// APPROVE
// ─────────────────────────────────────────────────────────────

/// Approve spender to spend up to `amount` on behalf of caller.
/// Args: 0: spender, 1: amount (0 to revoke)
#[no_mangle]
pub extern "C" fn approve() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    let spender = match arg(0) {
        Some(s) if !s.is_empty() => s,
        _ => return fail("spender address required"),
    };
    let amount = parse_u128(&arg(1).unwrap_or_default());
    let owner = caller();
    if owner == spender {
        return fail("cannot approve self");
    }

    set_allowance(&owner, &spender, amount);

    event::emit(
        "USP01:Approval",
        &format!(
            r#"{{"owner":"{}","spender":"{}","amount":"{}"}}"#,
            json_escape(&owner),
            json_escape(&spender),
            u128_to_str(amount)
        ),
    );

    set_return_str(&format!(
        r#"{{"success":true,"owner":"{}","spender":"{}","amount":"{}"}}"#,
        json_escape(&owner),
        json_escape(&spender),
        u128_to_str(amount)
    ));
    0
}

// ─────────────────────────────────────────────────────────────
// TRANSFER_FROM
// ─────────────────────────────────────────────────────────────

/// Transfer tokens from `from` to `to` using caller's allowance.
/// Args: 0: from, 1: to, 2: amount
#[no_mangle]
pub extern "C" fn transfer_from() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    let from = match arg(0) {
        Some(a) if !a.is_empty() => a,
        _ => return fail("from address required"),
    };
    let to = match arg(1) {
        Some(a) if !a.is_empty() => a,
        _ => return fail("to address required"),
    };
    let amount = parse_u128(&arg(2).unwrap_or_default());
    if amount == 0 {
        return fail("amount must be > 0");
    }
    if from == to {
        return fail("from and to must differ");
    }

    let spender = caller();
    let allowance = get_allowance(&from, &spender);
    if allowance < amount {
        return fail("allowance exceeded");
    }

    let from_bal = get_balance(&from);
    if from_bal < amount {
        return fail("insufficient balance");
    }

    // Debit owner
    set_balance(
        &from,
        match from_bal.checked_sub(amount) {
            Some(v) => v,
            None => return fail("arithmetic underflow"),
        },
    );

    // Credit recipient
    let to_bal = get_balance(&to);
    set_balance(
        &to,
        match to_bal.checked_add(amount) {
            Some(v) => v,
            None => return fail("arithmetic overflow"),
        },
    );

    // Reduce allowance
    set_allowance(&from, &spender, allowance.saturating_sub(amount));

    event::emit(
        "USP01:Transfer",
        &format!(
            r#"{{"from":"{}","to":"{}","amount":"{}","spender":"{}"}}"#,
            json_escape(&from),
            json_escape(&to),
            u128_to_str(amount),
            json_escape(&spender)
        ),
    );

    set_return_str(&format!(
        r#"{{"success":true,"from":"{}","to":"{}","amount":"{}"}}"#,
        json_escape(&from),
        json_escape(&to),
        u128_to_str(amount)
    ));
    0
}

// ─────────────────────────────────────────────────────────────
// BURN
// ─────────────────────────────────────────────────────────────

/// Burn tokens from caller's balance, reducing total supply.
/// Args: 0: amount
#[no_mangle]
pub extern "C" fn burn() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    let amount = parse_u128(&arg(0).unwrap_or_default());
    if amount == 0 {
        return fail("amount must be > 0");
    }
    let from = caller();
    let bal = get_balance(&from);
    if bal < amount {
        return fail("insufficient balance to burn");
    }

    set_balance(&from, bal.saturating_sub(amount));
    let new_supply = get_total_supply().saturating_sub(amount);
    set_total_supply(new_supply);

    event::emit(
        "USP01:Burn",
        &format!(
            r#"{{"from":"{}","amount":"{}","new_supply":"{}"}}"#,
            json_escape(&from),
            u128_to_str(amount),
            u128_to_str(new_supply)
        ),
    );

    set_return_str(&format!(
        r#"{{"success":true,"burned":"{}","new_supply":"{}"}}"#,
        u128_to_str(amount),
        u128_to_str(new_supply)
    ));
    0
}

// ─────────────────────────────────────────────────────────────
// READ-ONLY QUERIES
// ─────────────────────────────────────────────────────────────

/// Return balance of an account. Args: 0: account
#[no_mangle]
pub extern "C" fn balance_of() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    let account = match arg(0) {
        Some(a) if !a.is_empty() => a,
        _ => return fail("account address required"),
    };
    ok_data(&format!(
        r#"{{"account":"{}","balance":"{}"}}"#,
        json_escape(&account),
        u128_to_str(get_balance(&account))
    ))
}

/// Return allowance granted by owner to spender. Args: 0: owner, 1: spender
#[no_mangle]
pub extern "C" fn allowance_of() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    let owner = match arg(0) {
        Some(a) if !a.is_empty() => a,
        _ => return fail("owner address required"),
    };
    let spender = match arg(1) {
        Some(a) if !a.is_empty() => a,
        _ => return fail("spender address required"),
    };
    ok_data(&format!(
        r#"{{"owner":"{}","spender":"{}","allowance":"{}"}}"#,
        json_escape(&owner),
        json_escape(&spender),
        u128_to_str(get_allowance(&owner, &spender))
    ))
}

/// Return current total supply.
#[no_mangle]
pub extern "C" fn total_supply() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    ok_data(&format!(
        r#"{{"total_supply":"{}"}}"#,
        u128_to_str(get_total_supply())
    ))
}

/// Return complete token metadata.
#[no_mangle]
pub extern "C" fn token_info() -> i32 {
    if !is_initialized() {
        return fail("Contract not initialized");
    }
    let name = state::get_str("usp01:name").unwrap_or_default();
    let symbol = state::get_str("usp01:symbol").unwrap_or_default();
    let decimals = parse_u128(&state::get_str("usp01:decimals").unwrap_or_default());
    let owner = state::get_str("usp01:owner").unwrap_or_default();

    ok_data(&format!(
        r#"{{"name":"{}","symbol":"{}","decimals":{},"total_supply":"{}","owner":"{}","standard":"USP-01"}}"#,
        json_escape(&name),
        json_escape(&symbol),
        decimals,
        u128_to_str(get_total_supply()),
        json_escape(&owner)
    ))
}
