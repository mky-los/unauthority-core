#!/bin/zsh
set -e

CLI="./target/release/los-cli"
CONTRACT="LOSCon0478c27646ec544602c65c8e87e9cfd2"
WALLET="anggaran-demo"
export LOS_RPC_URL="https://los.yourmoonkey.com/api"
export LOS_WALLET_PASSWORD="demoanggaran2025"

call() {
    local fn="$1"; shift
    echo ">>> $fn $@"
    $CLI contract call -w $WALLET -c $CONTRACT -f "$fn" -a "$@" 2>&1 | grep -E '(✓|✗|success|error|Gas used|Block hash|"data")'
    echo ""
}

echo "=== Adding budget items ==="
call add_budget_item "1" "income" "A.1.02" "Pajak Pertambahan Nilai (PPN)" "Kementerian Keuangan" "750000000000000"
call add_budget_item "1" "income" "A.1.03" "Cukai Hasil Tembakau" "Dirjen Bea Cukai" "220000000000000"
call add_budget_item "1" "income" "A.2.01" "PNBP Sumber Daya Alam" "Kementerian ESDM" "450000000000000"
call add_budget_item "1" "income" "A.2.02" "Pendapatan BLU" "Kementerian Kesehatan" "182300000000000"
call add_budget_item "1" "income" "A.3.01" "Hibah Luar Negeri" "Kementerian Keuangan" "300000000000000"
call add_budget_item "1" "expense" "B.1.01" "Belanja Pegawai ASN" "Kementerian PAN-RB" "420000000000000"
call add_budget_item "1" "expense" "B.2.01" "Infrastruktur dan Konstruksi" "Kementerian PUPR" "600000000000000"
call add_budget_item "1" "expense" "B.2.02" "Dana Alokasi Khusus (DAK)" "Kementerian Keuangan" "380000000000000"
call add_budget_item "1" "expense" "B.3.01" "Program Bantuan Sosial" "Kementerian Sosial" "470000000000000"
call add_budget_item "1" "expense" "B.4.01" "Subsidi Energi dan Pupuk" "Kementerian ESDM" "350000000000000"
call add_budget_item "1" "expense" "B.5.01" "Belanja Pertahanan" "Kementerian Pertahanan" "320400000000000"

echo "=== Recording realizations ==="
call record_realization "1" "1" "410000000000000" "Penerimaan PPh Semester 1 2025" "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2"
call record_realization "1" "1" "402000000000000" "Penerimaan PPh Semester 2 2025" "b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3"
call record_realization "1" "2" "340000000000000" "Penerimaan PPN Semester 1" "c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4"
call record_realization "1" "2" "285000000000000" "Penerimaan PPN Semester 2" "d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5"
call record_realization "1" "7" "180000000000000" "Gaji ASN Semester 1 2025" "e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6"
call record_realization "1" "7" "185000000000000" "Gaji ASN Semester 2 2025" "f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7"
call record_realization "1" "8" "310000000000000" "Pembangunan Jalan Tol Trans Jawa" "a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8"
call record_realization "1" "10" "240000000000000" "PKH dan Sembako Jan-Sep 2025" "b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9"
call record_realization "1" "10" "140000000000000" "PKH dan Sembako Okt-Des 2025" "c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0"
call record_realization "1" "11" "120000000000000" "Subsidi BBM Semester 1" "d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1"
call record_realization "1" "11" "83200000000000" "Subsidi Pupuk Jan-Agu 2025" "e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2"
call record_realization "1" "11" "50000000000000" "Subsidi LPG 3kg - PERLU REVIEW" "f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3"
call record_realization "1" "12" "140000000000000" "Belanja Alutsista Semester 1" "a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0e1f2a3b4"

echo "=== Verifying ==="
$CLI contract call -w $WALLET -c $CONTRACT -f get_info
$CLI contract call -w $WALLET -c $CONTRACT -f get_summary -a "1"

echo "=== DONE ==="
