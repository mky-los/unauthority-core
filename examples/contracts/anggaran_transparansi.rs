// SPDX-License-Identifier: AGPL-3.0-only
//! # Sistem Transparansi Anggaran (Budget Transparency System)
//!
//! Smart contract untuk transparansi, validasi, dan keamanan data
//! anggaran belanja, pemasukan, dan realisasi anggaran dana.
//!
//! Dirancang untuk demo presentasi ke pemerintah Indonesia.
//!
//! ## Fitur
//! - Multi-role: Admin (pemerintah), Auditor (BPK/BPKP), Publik (masyarakat)
//! - Pencatatan anggaran pemasukan & belanja per tahun anggaran
//! - Realisasi anggaran dengan bukti hash (dokumen pendukung)
//! - Validasi oleh auditor independen
//! - Flagging item mencurigakan
//! - Audit trail immutable di blockchain
//! - Semua data bisa dibaca publik (transparansi penuh)
//!
//! ## State Layout
//! ```text
//! ag:init              → "1"
//! ag:name              → Contract name
//! ag:admin             → Admin address (deployer)
//! ag:created_at        → Timestamp
//!
//! ag:auditor_count     → Number of registered auditors
//! ag:auditor:{addr}    → "1" if address is an auditor
//!
//! ag:fy_count          → Number of fiscal years
//! ag:fy:{id}:name      → "APBN 2025"
//! ag:fy:{id}:year      → "2025"
//! ag:fy:{id}:status    → "draft|active|closed|audited"
//! ag:fy:{id}:created_at→ Timestamp
//! ag:fy:{id}:created_by→ Creator address
//!
//! ag:item_count:{fy}   → Items in this fiscal year
//! ag:item:{fy}:{id}:cat    → "income|expense"
//! ag:item:{fy}:{id}:code   → "A.1.01" (kode mata anggaran)
//! ag:item:{fy}:{id}:name   → "Pajak Penghasilan"
//! ag:item:{fy}:{id}:dept   → "Kementerian Keuangan"
//! ag:item:{fy}:{id}:budget → Amount (decimal string, in Rupiah)
//! ag:item:{fy}:{id}:realized → Cumulative realized amount
//! ag:item:{fy}:{id}:status → "normal|flagged"
//! ag:item:{fy}:{id}:flag_reason → Reason if flagged
//! ag:item:{fy}:{id}:created_by → Creator address
//! ag:item:{fy}:{id}:created_at → Timestamp
//!
//! ag:real_count:{fy}:{item} → Realizations for this item
//! ag:real:{fy}:{item}:{id}:amount   → Realization amount
//! ag:real:{fy}:{item}:{id}:desc     → Description
//! ag:real:{fy}:{item}:{id}:evidence → Blake3 hash of evidence doc
//! ag:real:{fy}:{item}:{id}:by       → Recorder address
//! ag:real:{fy}:{item}:{id}:at       → Timestamp
//! ag:real:{fy}:{item}:{id}:valid    → "0" or "1"
//! ag:real:{fy}:{item}:{id}:valid_by → Validator address
//! ag:real:{fy}:{item}:{id}:valid_at → Validation timestamp
//!
//! ag:log_count         → Total audit log entries
//! ag:log:{id}:action   → Action type
//! ag:log:{id}:actor    → Who performed it
//! ag:log:{id}:at       → Timestamp
//! ag:log:{id}:detail   → JSON details
//! ```
//!
//! ## Compilation
//! ```bash
//! cargo build --target wasm32-unknown-unknown --release \
//!     -p los-contract-examples --bin anggaran_transparansi --features sdk
//! ```

#![no_std]
#![no_main]

extern crate alloc;
extern crate los_sdk;

use alloc::format;
use alloc::string::String;
use los_sdk::*;

// ─────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────

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

fn u64_to_str(val: u64) -> String {
    u128_to_str(val as u128)
}

fn parse_u64(s: &str) -> u64 {
    let v = parse_u128(s);
    if v > u64::MAX as u128 {
        0
    } else {
        v as u64
    }
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(c),
        }
    }
    out
}

fn fail(msg: &str) -> i32 {
    set_return_str(&format!(r#"{{"success":false,"msg":"{}"}}"#, json_escape(msg)));
    1
}

fn ok(data: &str) -> i32 {
    set_return_str(&format!(r#"{{"success":true,"data":{}}}"#, data));
    0
}

fn is_initialized() -> bool {
    state::get_str("ag:init").map_or(false, |v| v == "1")
}

fn get_admin() -> String {
    state::get_str("ag:admin").unwrap_or_default()
}

fn is_admin(addr: &str) -> bool {
    get_admin() == addr
}

fn is_auditor(addr: &str) -> bool {
    state::get_str(&format!("ag:auditor:{}", addr)).map_or(false, |v| v == "1")
}

fn get_count(key: &str) -> u64 {
    parse_u64(&state::get_str(key).unwrap_or_default())
}

fn set_count(key: &str, val: u64) {
    state::set_str(key, &u64_to_str(val));
}

fn get_amount(key: &str) -> u128 {
    parse_u128(&state::get_str(key).unwrap_or_default())
}

fn set_amount(key: &str, val: u128) {
    state::set_str(key, &u128_to_str(val));
}

/// Append to audit log
fn audit_log(action: &str, actor: &str, detail: &str) {
    let count = get_count("ag:log_count");
    let id = count + 1;
    let ts = timestamp();
    state::set_str(&format!("ag:log:{}:action", id), action);
    state::set_str(&format!("ag:log:{}:actor", id), actor);
    state::set_str(&format!("ag:log:{}:at", id), &u64_to_str(ts));
    state::set_str(&format!("ag:log:{}:detail", id), detail);
    set_count("ag:log_count", id);
}

fn require_init() -> bool {
    if !is_initialized() {
        fail("Contract belum diinisialisasi");
        return false;
    }
    true
}

fn require_admin() -> Option<String> {
    if !require_init() {
        return None;
    }
    let who = caller();
    if !is_admin(&who) {
        fail("Akses ditolak: hanya admin");
        return None;
    }
    Some(who)
}

fn require_auditor() -> Option<String> {
    if !require_init() {
        return None;
    }
    let who = caller();
    if !is_auditor(&who) {
        fail("Akses ditolak: hanya auditor");
        return None;
    }
    Some(who)
}

fn get_fy_status(fy_id: u64) -> String {
    state::get_str(&format!("ag:fy:{}:status", fy_id)).unwrap_or_default()
}

// ─────────────────────────────────────────────────────────────
// INIT — Deploy contract
// ─────────────────────────────────────────────────────────────

/// Initialize the budget transparency system.
/// Args: 0: name (e.g., "Sistem Transparansi APBN")
#[no_mangle]
pub extern "C" fn init() -> i32 {
    if is_initialized() {
        return fail("Sudah diinisialisasi");
    }

    let name = match arg(0) {
        Some(n) if !n.is_empty() && n.len() <= 128 => n,
        _ => return fail("Nama sistem diperlukan (1-128 karakter)"),
    };

    let admin = caller();
    if admin.is_empty() {
        return fail("Alamat caller tidak tersedia");
    }

    state::set_str("ag:init", "1");
    state::set_str("ag:name", &name);
    state::set_str("ag:admin", &admin);
    state::set_str("ag:created_at", &u64_to_str(timestamp()));
    set_count("ag:auditor_count", 0);
    set_count("ag:fy_count", 0);
    set_count("ag:log_count", 0);

    event::emit("AG:Init", &format!(
        r#"{{"name":"{}","admin":"{}"}}"#,
        json_escape(&name), json_escape(&admin)
    ));

    audit_log("init", &admin, &format!(
        r#"{{"name":"{}"}}"#, json_escape(&name)
    ));

    ok(&format!(
        r#"{{"name":"{}","admin":"{}","contract":"{}"}}"#,
        json_escape(&name), json_escape(&admin), json_escape(&self_address())
    ))
}

// ─────────────────────────────────────────────────────────────
// AUDITOR MANAGEMENT
// ─────────────────────────────────────────────────────────────

/// Register an auditor address.
/// Args: 0: auditor_address
#[no_mangle]
pub extern "C" fn add_auditor() -> i32 {
    let admin = match require_admin() {
        Some(a) => a,
        None => return 1,
    };

    let addr = match arg(0) {
        Some(a) if a.starts_with("LOSW") && a.len() > 10 => a,
        _ => return fail("Alamat auditor tidak valid"),
    };

    if is_auditor(&addr) {
        return fail("Auditor sudah terdaftar");
    }

    state::set_str(&format!("ag:auditor:{}", addr), "1");
    let count = get_count("ag:auditor_count");
    set_count("ag:auditor_count", count + 1);

    event::emit("AG:AddAuditor", &format!(
        r#"{{"auditor":"{}","by":"{}"}}"#,
        json_escape(&addr), json_escape(&admin)
    ));

    audit_log("add_auditor", &admin, &format!(
        r#"{{"auditor":"{}"}}"#, json_escape(&addr)
    ));

    ok(&format!(r#"{{"auditor":"{}"}}"#, json_escape(&addr)))
}

/// Remove an auditor.
/// Args: 0: auditor_address
#[no_mangle]
pub extern "C" fn remove_auditor() -> i32 {
    let admin = match require_admin() {
        Some(a) => a,
        None => return 1,
    };

    let addr = match arg(0) {
        Some(a) if !a.is_empty() => a,
        _ => return fail("Alamat auditor diperlukan"),
    };

    if !is_auditor(&addr) {
        return fail("Bukan auditor terdaftar");
    }

    state::del(&format!("ag:auditor:{}", addr));
    let count = get_count("ag:auditor_count");
    if count > 0 {
        set_count("ag:auditor_count", count - 1);
    }

    event::emit("AG:RemoveAuditor", &format!(
        r#"{{"auditor":"{}","by":"{}"}}"#,
        json_escape(&addr), json_escape(&admin)
    ));

    audit_log("remove_auditor", &admin, &format!(
        r#"{{"auditor":"{}"}}"#, json_escape(&addr)
    ));

    ok(&format!(r#"{{"auditor":"{}","removed":true}}"#, json_escape(&addr)))
}

// ─────────────────────────────────────────────────────────────
// FISCAL YEAR MANAGEMENT
// ─────────────────────────────────────────────────────────────

/// Create a new fiscal year.
/// Args: 0: name (e.g., "APBN 2025"), 1: year (e.g., "2025")
#[no_mangle]
pub extern "C" fn create_fiscal_year() -> i32 {
    let admin = match require_admin() {
        Some(a) => a,
        None => return 1,
    };

    let name = match arg(0) {
        Some(n) if !n.is_empty() && n.len() <= 64 => n,
        _ => return fail("Nama tahun anggaran diperlukan (1-64 karakter)"),
    };

    let year = match arg(1) {
        Some(y) if y.len() == 4 && parse_u64(&y) >= 2000 && parse_u64(&y) <= 2100 => y,
        _ => return fail("Tahun harus 4 digit (2000-2100)"),
    };

    let count = get_count("ag:fy_count");
    let id = count + 1;
    let ts = timestamp();

    state::set_str(&format!("ag:fy:{}:name", id), &name);
    state::set_str(&format!("ag:fy:{}:year", id), &year);
    state::set_str(&format!("ag:fy:{}:status", id), "draft");
    state::set_str(&format!("ag:fy:{}:created_at", id), &u64_to_str(ts));
    state::set_str(&format!("ag:fy:{}:created_by", id), &admin);
    set_count(&format!("ag:item_count:{}", id), 0);
    set_count("ag:fy_count", id);

    event::emit("AG:CreateFY", &format!(
        r#"{{"id":{},"name":"{}","year":"{}"}}"#,
        id, json_escape(&name), json_escape(&year)
    ));

    audit_log("create_fy", &admin, &format!(
        r#"{{"fy_id":{},"name":"{}","year":"{}"}}"#,
        id, json_escape(&name), json_escape(&year)
    ));

    ok(&format!(
        r#"{{"fy_id":{},"name":"{}","year":"{}","status":"draft"}}"#,
        id, json_escape(&name), json_escape(&year)
    ))
}

/// Activate a fiscal year (draft → active).
/// Args: 0: fy_id
#[no_mangle]
pub extern "C" fn activate_fiscal_year() -> i32 {
    let admin = match require_admin() {
        Some(a) => a,
        None => return 1,
    };

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let status = get_fy_status(fy_id);
    if status != "draft" {
        return fail("Hanya tahun anggaran berstatus draft yang bisa diaktifkan");
    }

    state::set_str(&format!("ag:fy:{}:status", fy_id), "active");

    event::emit("AG:ActivateFY", &format!(r#"{{"fy_id":{}}}"#, fy_id));
    audit_log("activate_fy", &admin, &format!(r#"{{"fy_id":{}}}"#, fy_id));

    ok(&format!(r#"{{"fy_id":{},"status":"active"}}"#, fy_id))
}

/// Close a fiscal year (active → closed). No more changes allowed.
/// Args: 0: fy_id
#[no_mangle]
pub extern "C" fn close_fiscal_year() -> i32 {
    let admin = match require_admin() {
        Some(a) => a,
        None => return 1,
    };

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let status = get_fy_status(fy_id);
    if status != "active" {
        return fail("Hanya tahun anggaran aktif yang bisa ditutup");
    }

    state::set_str(&format!("ag:fy:{}:status", fy_id), "closed");

    event::emit("AG:CloseFY", &format!(r#"{{"fy_id":{}}}"#, fy_id));
    audit_log("close_fy", &admin, &format!(r#"{{"fy_id":{}}}"#, fy_id));

    ok(&format!(r#"{{"fy_id":{},"status":"closed"}}"#, fy_id))
}

// ─────────────────────────────────────────────────────────────
// BUDGET ITEMS
// ─────────────────────────────────────────────────────────────

/// Add a budget item (income or expense).
/// Args: 0: fy_id, 1: category ("income"|"expense"), 2: code, 3: name, 4: department, 5: budget_amount
#[no_mangle]
pub extern "C" fn add_budget_item() -> i32 {
    let admin = match require_admin() {
        Some(a) => a,
        None => return 1,
    };

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let status = get_fy_status(fy_id);
    if status != "draft" && status != "active" {
        return fail("Tahun anggaran sudah ditutup, tidak bisa menambah item");
    }

    let category = match arg(1) {
        Some(c) if c == "income" || c == "expense" => c,
        _ => return fail("Kategori harus 'income' atau 'expense'"),
    };

    let code = match arg(2) {
        Some(c) if !c.is_empty() && c.len() <= 20 => c,
        _ => return fail("Kode anggaran diperlukan (1-20 karakter)"),
    };

    let name = match arg(3) {
        Some(n) if !n.is_empty() && n.len() <= 128 => n,
        _ => return fail("Nama item diperlukan (1-128 karakter)"),
    };

    let department = match arg(4) {
        Some(d) if !d.is_empty() && d.len() <= 128 => d,
        _ => return fail("Departemen/kementerian diperlukan"),
    };

    let budget = parse_u128(&arg(5).unwrap_or_default());
    if budget == 0 {
        return fail("Jumlah anggaran harus > 0");
    }

    let count_key = format!("ag:item_count:{}", fy_id);
    let count = get_count(&count_key);
    let id = count + 1;
    let ts = timestamp();
    let prefix = format!("ag:item:{}:{}", fy_id, id);

    state::set_str(&format!("{}:cat", prefix), &category);
    state::set_str(&format!("{}:code", prefix), &code);
    state::set_str(&format!("{}:name", prefix), &name);
    state::set_str(&format!("{}:dept", prefix), &department);
    set_amount(&format!("{}:budget", prefix), budget);
    set_amount(&format!("{}:realized", prefix), 0);
    state::set_str(&format!("{}:status", prefix), "normal");
    state::set_str(&format!("{}:flag_reason", prefix), "");
    state::set_str(&format!("{}:created_by", prefix), &admin);
    state::set_str(&format!("{}:created_at", prefix), &u64_to_str(ts));
    set_count(&format!("ag:real_count:{}:{}", fy_id, id), 0);
    set_count(&count_key, id);

    event::emit("AG:AddItem", &format!(
        r#"{{"fy_id":{},"item_id":{},"cat":"{}","code":"{}","name":"{}","dept":"{}","budget":"{}"}}"#,
        fy_id, id, json_escape(&category), json_escape(&code),
        json_escape(&name), json_escape(&department), u128_to_str(budget)
    ));

    audit_log("add_item", &admin, &format!(
        r#"{{"fy_id":{},"item_id":{},"cat":"{}","code":"{}","name":"{}","budget":"{}"}}"#,
        fy_id, id, json_escape(&category), json_escape(&code),
        json_escape(&name), u128_to_str(budget)
    ));

    ok(&format!(
        r#"{{"fy_id":{},"item_id":{},"cat":"{}","code":"{}","name":"{}","dept":"{}","budget":"{}"}}"#,
        fy_id, id, json_escape(&category), json_escape(&code),
        json_escape(&name), json_escape(&department), u128_to_str(budget)
    ))
}

// ─────────────────────────────────────────────────────────────
// REALIZATION
// ─────────────────────────────────────────────────────────────

/// Record a budget realization (disbursement or income receipt).
/// Args: 0: fy_id, 1: item_id, 2: amount, 3: description, 4: evidence_hash
#[no_mangle]
pub extern "C" fn record_realization() -> i32 {
    let admin = match require_admin() {
        Some(a) => a,
        None => return 1,
    };

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let status = get_fy_status(fy_id);
    if status != "active" {
        return fail("Realisasi hanya bisa dicatat pada tahun anggaran aktif");
    }

    let item_id = parse_u64(&arg(1).unwrap_or_default());
    let item_count = get_count(&format!("ag:item_count:{}", fy_id));
    if item_id == 0 || item_id > item_count {
        return fail("ID item anggaran tidak valid");
    }

    let amount = parse_u128(&arg(2).unwrap_or_default());
    if amount == 0 {
        return fail("Jumlah realisasi harus > 0");
    }

    let description = match arg(3) {
        Some(d) if !d.is_empty() && d.len() <= 256 => d,
        _ => return fail("Deskripsi diperlukan (1-256 karakter)"),
    };

    let evidence_hash = match arg(4) {
        Some(h) if h.len() == 64 => h, // 32 bytes hex = 64 chars
        _ => return fail("Hash bukti dokumen diperlukan (64 karakter hex, blake3)"),
    };

    // Update cumulative realized amount
    let item_prefix = format!("ag:item:{}:{}", fy_id, item_id);
    let current_realized = get_amount(&format!("{}:realized", item_prefix));
    let new_realized = match current_realized.checked_add(amount) {
        Some(v) => v,
        None => return fail("Overflow pada jumlah realisasi"),
    };
    set_amount(&format!("{}:realized", item_prefix), new_realized);

    // Create realization record
    let count_key = format!("ag:real_count:{}:{}", fy_id, item_id);
    let count = get_count(&count_key);
    let id = count + 1;
    let ts = timestamp();
    let prefix = format!("ag:real:{}:{}:{}", fy_id, item_id, id);

    set_amount(&format!("{}:amount", prefix), amount);
    state::set_str(&format!("{}:desc", prefix), &description);
    state::set_str(&format!("{}:evidence", prefix), &evidence_hash);
    state::set_str(&format!("{}:by", prefix), &admin);
    state::set_str(&format!("{}:at", prefix), &u64_to_str(ts));
    state::set_str(&format!("{}:valid", prefix), "0");
    state::set_str(&format!("{}:valid_by", prefix), "");
    state::set_str(&format!("{}:valid_at", prefix), "0");
    set_count(&count_key, id);

    event::emit("AG:RecordRealization", &format!(
        r#"{{"fy_id":{},"item_id":{},"real_id":{},"amount":"{}","desc":"{}","evidence":"{}"}}"#,
        fy_id, item_id, id, u128_to_str(amount),
        json_escape(&description), json_escape(&evidence_hash)
    ));

    audit_log("record_realization", &admin, &format!(
        r#"{{"fy_id":{},"item_id":{},"real_id":{},"amount":"{}"}}"#,
        fy_id, item_id, id, u128_to_str(amount)
    ));

    ok(&format!(
        r#"{{"fy_id":{},"item_id":{},"real_id":{},"amount":"{}","total_realized":"{}"}}"#,
        fy_id, item_id, id, u128_to_str(amount), u128_to_str(new_realized)
    ))
}

// ─────────────────────────────────────────────────────────────
// AUDITOR FUNCTIONS
// ─────────────────────────────────────────────────────────────

/// Validate a realization entry (auditor only).
/// Args: 0: fy_id, 1: item_id, 2: realization_id
#[no_mangle]
pub extern "C" fn validate_realization() -> i32 {
    let auditor = match require_auditor() {
        Some(a) => a,
        None => return 1,
    };

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    let item_id = parse_u64(&arg(1).unwrap_or_default());
    let real_id = parse_u64(&arg(2).unwrap_or_default());

    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let item_count = get_count(&format!("ag:item_count:{}", fy_id));
    if item_id == 0 || item_id > item_count {
        return fail("ID item tidak valid");
    }

    let real_count = get_count(&format!("ag:real_count:{}:{}", fy_id, item_id));
    if real_id == 0 || real_id > real_count {
        return fail("ID realisasi tidak valid");
    }

    let prefix = format!("ag:real:{}:{}:{}", fy_id, item_id, real_id);
    let already_valid = state::get_str(&format!("{}:valid", prefix))
        .map_or(false, |v| v == "1");

    if already_valid {
        return fail("Realisasi sudah divalidasi sebelumnya");
    }

    let ts = timestamp();
    state::set_str(&format!("{}:valid", prefix), "1");
    state::set_str(&format!("{}:valid_by", prefix), &auditor);
    state::set_str(&format!("{}:valid_at", prefix), &u64_to_str(ts));

    event::emit("AG:ValidateRealization", &format!(
        r#"{{"fy_id":{},"item_id":{},"real_id":{},"auditor":"{}"}}"#,
        fy_id, item_id, real_id, json_escape(&auditor)
    ));

    audit_log("validate_realization", &auditor, &format!(
        r#"{{"fy_id":{},"item_id":{},"real_id":{}}}"#,
        fy_id, item_id, real_id
    ));

    ok(&format!(
        r#"{{"fy_id":{},"item_id":{},"real_id":{},"validated":true,"auditor":"{}"}}"#,
        fy_id, item_id, real_id, json_escape(&auditor)
    ))
}

/// Flag a budget item as suspicious (auditor only).
/// Args: 0: fy_id, 1: item_id, 2: reason
#[no_mangle]
pub extern "C" fn flag_item() -> i32 {
    let auditor = match require_auditor() {
        Some(a) => a,
        None => return 1,
    };

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    let item_id = parse_u64(&arg(1).unwrap_or_default());

    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let item_count = get_count(&format!("ag:item_count:{}", fy_id));
    if item_id == 0 || item_id > item_count {
        return fail("ID item tidak valid");
    }

    let reason = match arg(2) {
        Some(r) if !r.is_empty() && r.len() <= 256 => r,
        _ => return fail("Alasan flag diperlukan (1-256 karakter)"),
    };

    let prefix = format!("ag:item:{}:{}", fy_id, item_id);
    state::set_str(&format!("{}:status", prefix), "flagged");
    state::set_str(&format!("{}:flag_reason", prefix), &reason);

    event::emit("AG:FlagItem", &format!(
        r#"{{"fy_id":{},"item_id":{},"reason":"{}","auditor":"{}"}}"#,
        fy_id, item_id, json_escape(&reason), json_escape(&auditor)
    ));

    audit_log("flag_item", &auditor, &format!(
        r#"{{"fy_id":{},"item_id":{},"reason":"{}"}}"#,
        fy_id, item_id, json_escape(&reason)
    ));

    ok(&format!(
        r#"{{"fy_id":{},"item_id":{},"status":"flagged","reason":"{}"}}"#,
        fy_id, item_id, json_escape(&reason)
    ))
}

/// Auditor approves/audits a closed fiscal year (closed → audited).
/// Args: 0: fy_id
#[no_mangle]
pub extern "C" fn approve_fiscal_year() -> i32 {
    let auditor = match require_auditor() {
        Some(a) => a,
        None => return 1,
    };

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let status = get_fy_status(fy_id);
    if status != "closed" {
        return fail("Hanya tahun anggaran berstatus 'closed' yang bisa di-audit");
    }

    state::set_str(&format!("ag:fy:{}:status", fy_id), "audited");

    event::emit("AG:ApproveFY", &format!(
        r#"{{"fy_id":{},"auditor":"{}"}}"#,
        fy_id, json_escape(&auditor)
    ));

    audit_log("approve_fy", &auditor, &format!(
        r#"{{"fy_id":{}}}"#, fy_id
    ));

    ok(&format!(r#"{{"fy_id":{},"status":"audited","auditor":"{}"}}"#,
        fy_id, json_escape(&auditor)
    ))
}

// ─────────────────────────────────────────────────────────────
// READ FUNCTIONS (publik — siapa saja bisa baca)
// ─────────────────────────────────────────────────────────────

/// Get contract info.
#[no_mangle]
pub extern "C" fn get_info() -> i32 {
    if !require_init() { return 1; }

    let name = state::get_str("ag:name").unwrap_or_default();
    let admin = get_admin();
    let created_at = state::get_str("ag:created_at").unwrap_or_default();
    let fy_count = get_count("ag:fy_count");
    let auditor_count = get_count("ag:auditor_count");
    let log_count = get_count("ag:log_count");

    ok(&format!(
        r#"{{"name":"{}","admin":"{}","created_at":"{}","fy_count":{},"auditor_count":{},"log_count":{},"contract":"{}"}}"#,
        json_escape(&name), json_escape(&admin), created_at,
        fy_count, auditor_count, log_count, json_escape(&self_address())
    ))
}

/// Get fiscal year details with budget summary.
/// Args: 0: fy_id
#[no_mangle]
pub extern "C" fn get_fiscal_year() -> i32 {
    if !require_init() { return 1; }

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let name = state::get_str(&format!("ag:fy:{}:name", fy_id)).unwrap_or_default();
    let year = state::get_str(&format!("ag:fy:{}:year", fy_id)).unwrap_or_default();
    let status = get_fy_status(fy_id);
    let created_at = state::get_str(&format!("ag:fy:{}:created_at", fy_id)).unwrap_or_default();
    let created_by = state::get_str(&format!("ag:fy:{}:created_by", fy_id)).unwrap_or_default();
    let item_count = get_count(&format!("ag:item_count:{}", fy_id));

    // Calculate totals
    let mut total_income_budget: u128 = 0;
    let mut total_expense_budget: u128 = 0;
    let mut total_income_realized: u128 = 0;
    let mut total_expense_realized: u128 = 0;
    let mut flagged_count: u64 = 0;

    let mut i: u64 = 1;
    while i <= item_count {
        let prefix = format!("ag:item:{}:{}", fy_id, i);
        let cat = state::get_str(&format!("{}:cat", prefix)).unwrap_or_default();
        let budget = get_amount(&format!("{}:budget", prefix));
        let realized = get_amount(&format!("{}:realized", prefix));
        let item_status = state::get_str(&format!("{}:status", prefix)).unwrap_or_default();

        if cat == "income" {
            total_income_budget = total_income_budget.saturating_add(budget);
            total_income_realized = total_income_realized.saturating_add(realized);
        } else {
            total_expense_budget = total_expense_budget.saturating_add(budget);
            total_expense_realized = total_expense_realized.saturating_add(realized);
        }

        if item_status == "flagged" {
            flagged_count += 1;
        }
        i += 1;
    }

    ok(&format!(
        concat!(
            r#"{{"fy_id":{},"name":"{}","year":"{}","status":"{}","#,
            r#""created_at":"{}","created_by":"{}","item_count":{},"flagged_count":{},"#,
            r#""total_income_budget":"{}","total_income_realized":"{}","#,
            r#""total_expense_budget":"{}","total_expense_realized":"{}"}}"#
        ),
        fy_id, json_escape(&name), json_escape(&year), json_escape(&status),
        created_at, json_escape(&created_by), item_count, flagged_count,
        u128_to_str(total_income_budget), u128_to_str(total_income_realized),
        u128_to_str(total_expense_budget), u128_to_str(total_expense_realized)
    ))
}

/// Get all budget items for a fiscal year.
/// Args: 0: fy_id
#[no_mangle]
pub extern "C" fn get_budget_items() -> i32 {
    if !require_init() { return 1; }

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let item_count = get_count(&format!("ag:item_count:{}", fy_id));
    let mut items = String::from("[");

    let mut i: u64 = 1;
    while i <= item_count {
        if i > 1 { items.push(','); }
        let prefix = format!("ag:item:{}:{}", fy_id, i);
        let cat = state::get_str(&format!("{}:cat", prefix)).unwrap_or_default();
        let code = state::get_str(&format!("{}:code", prefix)).unwrap_or_default();
        let name = state::get_str(&format!("{}:name", prefix)).unwrap_or_default();
        let dept = state::get_str(&format!("{}:dept", prefix)).unwrap_or_default();
        let budget = state::get_str(&format!("{}:budget", prefix)).unwrap_or_default();
        let realized = state::get_str(&format!("{}:realized", prefix)).unwrap_or_default();
        let item_status = state::get_str(&format!("{}:status", prefix)).unwrap_or_default();
        let flag_reason = state::get_str(&format!("{}:flag_reason", prefix)).unwrap_or_default();
        let real_count = get_count(&format!("ag:real_count:{}:{}", fy_id, i));

        items.push_str(&format!(
            concat!(
                r#"{{"id":{},"cat":"{}","code":"{}","name":"{}","dept":"{}","#,
                r#""budget":"{}","realized":"{}","status":"{}","flag_reason":"{}","realization_count":{}}}"#
            ),
            i, json_escape(&cat), json_escape(&code), json_escape(&name),
            json_escape(&dept), budget, realized, json_escape(&item_status),
            json_escape(&flag_reason), real_count
        ));
        i += 1;
    }
    items.push(']');

    ok(&format!(r#"{{"fy_id":{},"item_count":{},"items":{}}}"#,
        fy_id, item_count, items
    ))
}

/// Get a single budget item detail.
/// Args: 0: fy_id, 1: item_id
#[no_mangle]
pub extern "C" fn get_budget_item() -> i32 {
    if !require_init() { return 1; }

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    let item_id = parse_u64(&arg(1).unwrap_or_default());

    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }
    let item_count = get_count(&format!("ag:item_count:{}", fy_id));
    if item_id == 0 || item_id > item_count {
        return fail("ID item tidak valid");
    }

    let prefix = format!("ag:item:{}:{}", fy_id, item_id);
    let cat = state::get_str(&format!("{}:cat", prefix)).unwrap_or_default();
    let code = state::get_str(&format!("{}:code", prefix)).unwrap_or_default();
    let name = state::get_str(&format!("{}:name", prefix)).unwrap_or_default();
    let dept = state::get_str(&format!("{}:dept", prefix)).unwrap_or_default();
    let budget = state::get_str(&format!("{}:budget", prefix)).unwrap_or_default();
    let realized = state::get_str(&format!("{}:realized", prefix)).unwrap_or_default();
    let item_status = state::get_str(&format!("{}:status", prefix)).unwrap_or_default();
    let flag_reason = state::get_str(&format!("{}:flag_reason", prefix)).unwrap_or_default();
    let created_by = state::get_str(&format!("{}:created_by", prefix)).unwrap_or_default();
    let created_at = state::get_str(&format!("{}:created_at", prefix)).unwrap_or_default();

    ok(&format!(
        concat!(
            r#"{{"fy_id":{},"item_id":{},"cat":"{}","code":"{}","name":"{}","dept":"{}","#,
            r#""budget":"{}","realized":"{}","status":"{}","flag_reason":"{}","#,
            r#""created_by":"{}","created_at":"{}"}}"#
        ),
        fy_id, item_id, json_escape(&cat), json_escape(&code), json_escape(&name),
        json_escape(&dept), budget, realized, json_escape(&item_status),
        json_escape(&flag_reason), json_escape(&created_by), created_at
    ))
}

/// Get realizations for a budget item.
/// Args: 0: fy_id, 1: item_id
#[no_mangle]
pub extern "C" fn get_realizations() -> i32 {
    if !require_init() { return 1; }

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    let item_id = parse_u64(&arg(1).unwrap_or_default());

    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }
    let item_count = get_count(&format!("ag:item_count:{}", fy_id));
    if item_id == 0 || item_id > item_count {
        return fail("ID item tidak valid");
    }

    let real_count = get_count(&format!("ag:real_count:{}:{}", fy_id, item_id));
    let mut reals = String::from("[");

    let mut i: u64 = 1;
    while i <= real_count {
        if i > 1 { reals.push(','); }
        let prefix = format!("ag:real:{}:{}:{}", fy_id, item_id, i);
        let amount = state::get_str(&format!("{}:amount", prefix)).unwrap_or_default();
        let desc = state::get_str(&format!("{}:desc", prefix)).unwrap_or_default();
        let evidence = state::get_str(&format!("{}:evidence", prefix)).unwrap_or_default();
        let by = state::get_str(&format!("{}:by", prefix)).unwrap_or_default();
        let at = state::get_str(&format!("{}:at", prefix)).unwrap_or_default();
        let valid = state::get_str(&format!("{}:valid", prefix)).unwrap_or_default();
        let valid_by = state::get_str(&format!("{}:valid_by", prefix)).unwrap_or_default();
        let valid_at = state::get_str(&format!("{}:valid_at", prefix)).unwrap_or_default();

        reals.push_str(&format!(
            concat!(
                r#"{{"id":{},"amount":"{}","desc":"{}","evidence":"{}","#,
                r#""recorded_by":"{}","recorded_at":"{}","#,
                r#""validated":{},"validated_by":"{}","validated_at":"{}"}}"#
            ),
            i, amount, json_escape(&desc), json_escape(&evidence),
            json_escape(&by), at,
            if valid == "1" { "true" } else { "false" },
            json_escape(&valid_by), valid_at
        ));
        i += 1;
    }
    reals.push(']');

    ok(&format!(
        r#"{{"fy_id":{},"item_id":{},"realization_count":{},"realizations":{}}}"#,
        fy_id, item_id, real_count, reals
    ))
}

/// Get complete budget vs realization summary for a fiscal year.
/// Args: 0: fy_id
#[no_mangle]
pub extern "C" fn get_summary() -> i32 {
    if !require_init() { return 1; }

    let fy_id = parse_u64(&arg(0).unwrap_or_default());
    if fy_id == 0 || fy_id > get_count("ag:fy_count") {
        return fail("ID tahun anggaran tidak valid");
    }

    let name = state::get_str(&format!("ag:fy:{}:name", fy_id)).unwrap_or_default();
    let year = state::get_str(&format!("ag:fy:{}:year", fy_id)).unwrap_or_default();
    let status = get_fy_status(fy_id);
    let item_count = get_count(&format!("ag:item_count:{}", fy_id));

    // Build per-department aggregation
    let mut income_items = String::from("[");
    let mut expense_items = String::from("[");
    let mut income_first = true;
    let mut expense_first = true;

    let mut total_income_budget: u128 = 0;
    let mut total_expense_budget: u128 = 0;
    let mut total_income_realized: u128 = 0;
    let mut total_expense_realized: u128 = 0;
    let mut total_validated: u64 = 0;
    let mut total_realizations: u64 = 0;
    let mut flagged_count: u64 = 0;

    let mut i: u64 = 1;
    while i <= item_count {
        let prefix = format!("ag:item:{}:{}", fy_id, i);
        let cat = state::get_str(&format!("{}:cat", prefix)).unwrap_or_default();
        let code = state::get_str(&format!("{}:code", prefix)).unwrap_or_default();
        let item_name = state::get_str(&format!("{}:name", prefix)).unwrap_or_default();
        let dept = state::get_str(&format!("{}:dept", prefix)).unwrap_or_default();
        let budget = get_amount(&format!("{}:budget", prefix));
        let realized = get_amount(&format!("{}:realized", prefix));
        let item_status = state::get_str(&format!("{}:status", prefix)).unwrap_or_default();

        // Count realizations and validations for this item
        let real_count = get_count(&format!("ag:real_count:{}:{}", fy_id, i));
        let mut validated: u64 = 0;
        let mut j: u64 = 1;
        while j <= real_count {
            let v = state::get_str(&format!("ag:real:{}:{}:{}:valid", fy_id, i, j)).unwrap_or_default();
            if v == "1" { validated += 1; }
            j += 1;
        }

        total_realizations += real_count;
        total_validated += validated;
        if item_status == "flagged" { flagged_count += 1; }

        let entry = format!(
            r#"{{"id":{},"code":"{}","name":"{}","dept":"{}","budget":"{}","realized":"{}","status":"{}","realization_count":{},"validated_count":{}}}"#,
            i, json_escape(&code), json_escape(&item_name), json_escape(&dept),
            u128_to_str(budget), u128_to_str(realized), json_escape(&item_status),
            real_count, validated
        );

        if cat == "income" {
            if !income_first { income_items.push(','); }
            income_items.push_str(&entry);
            income_first = false;
            total_income_budget = total_income_budget.saturating_add(budget);
            total_income_realized = total_income_realized.saturating_add(realized);
        } else {
            if !expense_first { expense_items.push(','); }
            expense_items.push_str(&entry);
            expense_first = false;
            total_expense_budget = total_expense_budget.saturating_add(budget);
            total_expense_realized = total_expense_realized.saturating_add(realized);
        }
        i += 1;
    }
    income_items.push(']');
    expense_items.push(']');

    ok(&format!(
        concat!(
            r#"{{"fy_id":{},"name":"{}","year":"{}","status":"{}","item_count":{},"#,
            r#""flagged_count":{},"total_realizations":{},"total_validated":{},"#,
            r#""total_income_budget":"{}","total_income_realized":"{}","#,
            r#""total_expense_budget":"{}","total_expense_realized":"{}","#,
            r#""income_items":{},"expense_items":{}}}"#
        ),
        fy_id, json_escape(&name), json_escape(&year), json_escape(&status),
        item_count, flagged_count, total_realizations, total_validated,
        u128_to_str(total_income_budget), u128_to_str(total_income_realized),
        u128_to_str(total_expense_budget), u128_to_str(total_expense_realized),
        income_items, expense_items
    ))
}

/// Get audit log entries.
/// Args: 0: offset (default 0), 1: limit (default 20, max 50)
#[no_mangle]
pub extern "C" fn get_audit_log() -> i32 {
    if !require_init() { return 1; }

    let total = get_count("ag:log_count");
    let offset = parse_u64(&arg(0).unwrap_or_default());
    let mut limit = parse_u64(&arg(1).unwrap_or_default());
    if limit == 0 { limit = 20; }
    if limit > 50 { limit = 50; }

    let mut entries = String::from("[");
    let mut first = true;

    // Return most recent first
    if total > 0 {
        let start = if total > offset { total - offset } else { 0 };
        let mut remaining = limit;
        let mut idx = start;

        while idx > 0 && remaining > 0 {
            if !first { entries.push(','); }
            let action = state::get_str(&format!("ag:log:{}:action", idx)).unwrap_or_default();
            let actor = state::get_str(&format!("ag:log:{}:actor", idx)).unwrap_or_default();
            let at = state::get_str(&format!("ag:log:{}:at", idx)).unwrap_or_default();
            let detail = state::get_str(&format!("ag:log:{}:detail", idx)).unwrap_or_default();

            entries.push_str(&format!(
                r#"{{"id":{},"action":"{}","actor":"{}","timestamp":"{}","detail":{}}}"#,
                idx, json_escape(&action), json_escape(&actor), at, detail
            ));
            first = false;
            idx -= 1;
            remaining -= 1;
        }
    }
    entries.push(']');

    ok(&format!(
        r#"{{"total":{},"offset":{},"limit":{},"entries":{}}}"#,
        total, offset, limit, entries
    ))
}

/// Verify document hash against stored evidence.
/// Args: 0: evidence_hash (64-char hex)
/// Returns all realizations that match this evidence hash.
#[no_mangle]
pub extern "C" fn verify_evidence() -> i32 {
    if !require_init() { return 1; }

    let hash = match arg(0) {
        Some(h) if h.len() == 64 => h,
        _ => return fail("Hash bukti diperlukan (64 karakter hex, blake3)"),
    };

    let fy_count = get_count("ag:fy_count");
    let mut matches = String::from("[");
    let mut first = true;

    let mut fy: u64 = 1;
    while fy <= fy_count {
        let item_count = get_count(&format!("ag:item_count:{}", fy));
        let mut item: u64 = 1;
        while item <= item_count {
            let real_count = get_count(&format!("ag:real_count:{}:{}", fy, item));
            let mut r: u64 = 1;
            while r <= real_count {
                let evidence = state::get_str(&format!("ag:real:{}:{}:{}:evidence", fy, item, r))
                    .unwrap_or_default();
                if evidence == hash {
                    if !first { matches.push(','); }
                    let amount = state::get_str(&format!("ag:real:{}:{}:{}:amount", fy, item, r))
                        .unwrap_or_default();
                    let valid = state::get_str(&format!("ag:real:{}:{}:{}:valid", fy, item, r))
                        .unwrap_or_default();
                    matches.push_str(&format!(
                        r#"{{"fy_id":{},"item_id":{},"real_id":{},"amount":"{}","validated":{}}}"#,
                        fy, item, r, amount,
                        if valid == "1" { "true" } else { "false" }
                    ));
                    first = false;
                }
                r += 1;
            }
            item += 1;
        }
        fy += 1;
    }
    matches.push(']');

    ok(&format!(
        r#"{{"hash":"{}","found":{},"matches":{}}}"#,
        json_escape(&hash), !first, matches
    ))
}
