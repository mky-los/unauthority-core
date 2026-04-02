#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────
# Deploy & Initialize: Sistem Transparansi Anggaran
#
# Script ini:
#   1. Compile smart contract ke WASM
#   2. Deploy ke node LoS
#   3. Initialize contract
#   4. Setup demo data (auditor, tahun anggaran, item, realisasi)
#
# Usage:
#   ./deploy_demo.sh [NODE_URL] [CALLER_ADDRESS]
#
# Default:
#   NODE_URL=http://localhost:3030
#   CALLER_ADDRESS=LOSWDemoAdmin (testnet/dev only)
# ──────────────────────────────────────────────────────────
set -euo pipefail

NODE_URL="${1:-http://localhost:3030}"
CALLER="${2:-LOSWDemoAdmin}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
WASM_PATH="$ROOT_DIR/target/wasm32-unknown-unknown/release/anggaran_transparansi.wasm"

echo "══════════════════════════════════════════════════════"
echo "  Sistem Transparansi Anggaran — Deploy & Demo Setup"
echo "══════════════════════════════════════════════════════"
echo ""
echo "  Node:   $NODE_URL"
echo "  Caller: $CALLER"
echo ""

# ─── Step 1: Compile ────────────────────────────────────
echo "▶ [1/7] Compiling smart contract ke WASM..."
cd "$ROOT_DIR"
cargo build --target wasm32-unknown-unknown --release \
    -p los-contract-examples --bin anggaran_transparansi --features sdk 2>&1 | tail -3

if [ ! -f "$WASM_PATH" ]; then
    echo "❌ WASM file not found: $WASM_PATH"
    exit 1
fi

WASM_SIZE=$(wc -c < "$WASM_PATH" | tr -d ' ')
echo "   ✅ Compiled: $WASM_PATH ($WASM_SIZE bytes)"

# ─── Step 2: Deploy ─────────────────────────────────────
echo ""
echo "▶ [2/7] Deploying contract..."
BYTECODE=$(base64 < "$WASM_PATH")

DEPLOY_RES=$(curl -s -X POST "$NODE_URL/deploy-contract" \
    -H "Content-Type: application/json" \
    -d "{
        \"owner\": \"$CALLER\",
        \"bytecode\": \"$BYTECODE\",
        \"initial_state\": {},
        \"amount_cil\": 0
    }")

CONTRACT=$(echo "$DEPLOY_RES" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('contract_address',''))" 2>/dev/null || echo "")

if [ -z "$CONTRACT" ]; then
    echo "❌ Deploy failed:"
    echo "$DEPLOY_RES"
    exit 1
fi
echo "   ✅ Contract: $CONTRACT"

# ─── Helper: call contract ──────────────────────────────
call_contract() {
    local fn="$1"
    shift
    local args="["
    local first=true
    for a in "$@"; do
        if [ "$first" = true ]; then first=false; else args+=","; fi
        args+="\"$a\""
    done
    args+="]"

    local result
    result=$(curl -s -X POST "$NODE_URL/call-contract" \
        -H "Content-Type: application/json" \
        -d "{
            \"contract_address\": \"$CONTRACT\",
            \"function\": \"$fn\",
            \"args\": $args,
            \"gas_limit\": 5000000,
            \"caller\": \"$CALLER\",
            \"amount_cil\": 0
        }")
    echo "$result"
}

# ─── Step 3: Initialize ─────────────────────────────────
echo ""
echo "▶ [3/7] Initializing contract..."
RES=$(call_contract "init" "Sistem Transparansi APBN Republik Indonesia")
echo "   $(echo "$RES" | python3 -c "import sys,json; d=json.load(sys.stdin); r=json.loads(d.get('result',{}).get('output','{}')); print('✅' if r.get('success') else '❌', json.dumps(r.get('data',r.get('msg','?')),ensure_ascii=False))" 2>/dev/null || echo "$RES")"

# ─── Step 4: Add Auditors ───────────────────────────────
echo ""
echo "▶ [4/7] Registering auditors..."
call_contract "add_auditor" "LOSWAuditorBPK001abc" > /dev/null
echo "   ✅ Auditor 1: LOSWAuditorBPK001abc (BPK RI)"
call_contract "add_auditor" "LOSWAuditorBPKP02xyz" > /dev/null
echo "   ✅ Auditor 2: LOSWAuditorBPKP02xyz (BPKP)"

# ─── Step 5: Create Fiscal Year ─────────────────────────
echo ""
echo "▶ [5/7] Creating fiscal year APBN 2025..."
call_contract "create_fiscal_year" "APBN Tahun Anggaran 2025" "2025" > /dev/null
echo "   ✅ Tahun Anggaran created (ID: 1)"

call_contract "activate_fiscal_year" "1" > /dev/null
echo "   ✅ Status: active"

# ─── Step 6: Add Budget Items ───────────────────────────
echo ""
echo "▶ [6/7] Adding budget items..."

# Income items
items=(
    "income|A.1.01|Pajak Penghasilan (PPh)|Kementerian Keuangan|900000000000000"
    "income|A.1.02|Pajak Pertambahan Nilai (PPN)|Kementerian Keuangan|750000000000000"
    "income|A.1.03|Cukai Hasil Tembakau|Direktorat Jenderal Bea dan Cukai|220000000000000"
    "income|A.2.01|Penerimaan Negara Bukan Pajak (PNBP)|Kementerian ESDM|450000000000000"
    "income|A.2.02|Pendapatan BLU|Kementerian Kesehatan|182300000000000"
    "income|A.3.01|Hibah Luar Negeri|Kementerian Keuangan|300000000000000"
    "expense|B.1.01|Belanja Pegawai ASN|Kementerian PAN-RB|420000000000000"
    "expense|B.2.01|Infrastruktur dan Konstruksi|Kementerian PUPR|600000000000000"
    "expense|B.2.02|Dana Alokasi Khusus (DAK)|Kementerian Keuangan|380000000000000"
    "expense|B.3.01|Program Bantuan Sosial|Kementerian Sosial|470000000000000"
    "expense|B.4.01|Subsidi Energi dan Pupuk|Kementerian ESDM|350000000000000"
    "expense|B.5.01|Belanja Pertahanan|Kementerian Pertahanan|320400000000000"
)

item_id=0
for item_str in "${items[@]}"; do
    IFS='|' read -r cat code name dept budget <<< "$item_str"
    call_contract "add_budget_item" "1" "$cat" "$code" "$name" "$dept" "$budget" > /dev/null
    item_id=$((item_id + 1))
    echo "   ✅ [$item_id] $code - $name ($cat: Rp $budget)"
done

# ─── Step 7: Record Sample Realizations ─────────────────
echo ""
echo "▶ [7/7] Recording sample realizations..."

# Item 1 (PPh) — 2 realizations
call_contract "record_realization" "1" "1" "410000000000000" "Penerimaan PPh Semester 1 2025" "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2" > /dev/null
echo "   ✅ Realisasi: PPh Semester 1 — Rp 410T"
call_contract "record_realization" "1" "1" "402000000000000" "Penerimaan PPh Semester 2 2025" "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3" > /dev/null
echo "   ✅ Realisasi: PPh Semester 2 — Rp 402T"

# Item 7 (Belanja Pegawai) — 2 realizations
call_contract "record_realization" "1" "7" "180000000000000" "Gaji ASN Semester 1" "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4" > /dev/null
echo "   ✅ Realisasi: Gaji ASN Semester 1 — Rp 180T"
call_contract "record_realization" "1" "7" "185000000000000" "Gaji ASN Semester 2" "d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5" > /dev/null
echo "   ✅ Realisasi: Gaji ASN Semester 2 — Rp 185T"

# Item 8 (Infrastruktur) — 1 realization
call_contract "record_realization" "1" "8" "310000000000000" "Pembangunan Jalan Tol Trans Jawa Lanjutan" "e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6" > /dev/null
echo "   ✅ Realisasi: Infrastruktur — Rp 310T"

# Item 10 (Bansos) — 1 realization
call_contract "record_realization" "1" "10" "240000000000000" "PKH dan Sembako Jan-Sep 2025" "f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7" > /dev/null
echo "   ✅ Realisasi: Bansos PKH — Rp 240T"

echo ""
echo "══════════════════════════════════════════════════════"
echo "  ✅ Deploy & Demo Setup Complete!"
echo "══════════════════════════════════════════════════════"
echo ""
echo "  Contract Address: $CONTRACT"
echo "  Node URL:         $NODE_URL"
echo ""
echo "  Untuk membuka demo web app:"
echo "    open examples/demo-anggaran/index.html"
echo ""
echo "  Lalu masukkan:"
echo "    Node URL: $NODE_URL"
echo "    Contract: $CONTRACT"
echo "    Uncheck 'Mode Demo' untuk menggunakan data live"
echo ""
echo "  API endpoints:"
echo "    GET  $NODE_URL/contract/$CONTRACT"
echo "    POST $NODE_URL/call-contract"
echo "══════════════════════════════════════════════════════"
