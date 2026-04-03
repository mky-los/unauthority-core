# API Reference — Unauthority (LOS) v2.3.1

Complete REST API and gRPC API documentation for the `los-node` validator binary.

---

## Base URL

| Protocol | Address | Notes |
|---|---|---|
| **REST** | `http://127.0.0.1:3030` | Default port, configurable via `--port` |
| **gRPC** | `127.0.0.1:23030` | Always REST port + 20,000 |
| **Tor** | `http://YOUR_ONION.onion:3030` | Via SOCKS5 proxy |

## Authentication

No authentication required. Rate limiting is enforced per IP for state-changing endpoints.

## Error Format

All errors return:
```json
{ "status": "error", "msg": "Description of the error", "code": 400 }
```

---

## Table of Contents

- [Status Endpoints](#status-endpoints)
- [Account Endpoints](#account-endpoints)
- [Block Endpoints](#block-endpoints)
- [Transaction Endpoints](#transaction-endpoints)
- [Validator Endpoints](#validator-endpoints)
- [Consensus](#consensus)
- [Smart Contract Endpoints](#smart-contract-endpoints)
- [Network Endpoints](#network-endpoints)
- [Wallet Endpoints](#wallet-endpoints)
- [Utility Endpoints](#utility-endpoints)
- [gRPC API](#grpc-api)
- [USP-01 Token Endpoints](#usp-01-token-endpoints)
- [DEX AMM Endpoints](#dex-amm-endpoints)
- [CLI Reference](#cli-reference)
- [Rate Limits](#rate-limits)

---

## Status Endpoints

### GET `/`

Node status overview with all available endpoints.

**Response:**
```json
{
  "name": "Unauthority (LOS) Blockchain API",
  "version": "2.2.0",
  "network": "mainnet",
  "status": "operational",
  "description": "Decentralized blockchain with aBFT consensus",
  "endpoints": {
    "health": "GET /health - Health check",
    "supply": "GET /supply - Total supply and remaining",
    "bal": "GET /bal/{address} - Account balance (short alias)",
    "send": "POST /send {from, target, amount} - Send transaction",
    "...": "..."
  }
}
```

### GET `/health`

Health check for monitoring and load balancing.

**Response:**
```json
{
  "status": "healthy",
  "version": "2.2.0",
  "timestamp": 1771277598,
  "uptime_seconds": 86400,
  "chain": {
    "accounts": 8,
    "blocks": 42,
    "id": "los-mainnet"
  },
  "database": {
    "accounts_count": 8,
    "blocks_count": 42,
    "size_on_disk": 524287
  }
}
```

### GET `/node-info`

Detailed node information.

**Response:**
```json
{
  "node_id": "validator-1",
  "version": "2.2.0",
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "block_count": 42,
  "account_count": 8,
  "peers": 4,
  "is_validator": true,
  "uptime_seconds": 86400,
  "network": "mainnet"
}
```

### GET `/supply`

Total supply and remaining supply information.

**Response:**
```json
{
  "total_supply": "21936236.00000000000",
  "total_supply_cil": 2193623600000000000,
  "circulating_supply": "777823.00000000000",
  "circulating_supply_cil": 77782300000000000,
  "remaining_supply": "21158413.00000000000",
  "remaining_supply_cil": 2115841300000000000
}
```

### GET `/metrics`

Prometheus-compatible metrics output.

**Response:** (text/plain)
```
# HELP los_blocks_total Total blocks in ledger
los_blocks_total 42
# HELP los_accounts_total Total accounts
los_accounts_total 8
# HELP los_active_validators Active validator count
los_active_validators 4
# HELP los_peer_count Connected peers
los_peer_count 4
# HELP los_consensus_rounds aBFT consensus rounds
los_consensus_rounds 128
# HELP los_uptime_seconds Node uptime
los_uptime_seconds 86400
```

---

## Account Endpoints

### GET `/bal/{address}`

Get account balance. Returns balance in both CIL (atomic unit) and LOS.

**Example:** `GET /bal/LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1`

**Response:**
```json
{
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "balance_cil": 100000000000000,
  "balance_cil_str": "100000000000000",
  "balance_los": "1000.00000000000",
  "block_count": 0,
  "head": "0"
}
```

### GET `/balance/{address}`

Alias for `/bal/{address}`. Same response format.

### GET `/account/{address}`

Full account details including balance, block count, validator status, and recent transaction history.

**Example:** `GET /account/LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1`

**Response:**
```json
{
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "balance_cil": 100000000000000,
  "balance_los": "1000.00000000000",
  "block_count": 5,
  "head": "abc123...",
  "is_validator": true,
  "stake_cil": 100000000000000,
  "recent_blocks": [ ... ]
}
```

### GET `/history/{address}`

Transaction history for an address.

**Example:** `GET /history/LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1`

**Response:**
```json
{
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "transactions": [
    {
      "hash": "abc123...",
      "type": "Send",
      "amount": 100000000000000,
      "from": "LOSX7dSt...",
      "to": "LOSWoNus...",
      "timestamp": 1771277598,
      "fee": 100000000
    }
  ]
}
```

### GET `/fee-estimate/{address}`

Estimate the transaction fee for an address. Returns the flat BASE_FEE_CIL.

**Example:** `GET /fee-estimate/LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1`

**Response:**
```json
{
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "fee_cil": 100000000,
  "fee_los": "0.00100000000"
}
```

---

## Block Endpoints

### GET `/block`

Latest block across all accounts.

**Response:**
```json
{
  "account": "LOSX7dSt...",
  "previous": "def456...",
  "block_type": "Send",
  "amount": 50000000000000,
  "link": "LOSWoNus...",
  "hash": "abc123...",
  "timestamp": 1771277598,
  "height": 42
}
```

### GET `/block/{hash}`

Get a specific block by its SHA-3 hash.

**Example:** `GET /block/abc123def456...`

### GET `/blocks/recent`

Recent blocks (last 50).

**Response:**
```json
{
  "blocks": [ ... ],
  "count": 50
}
```

---

## Transaction Endpoints

### POST `/send`

Send LOS to another address.

> **MAINNET: Signature is REQUIRED.** On mainnet, every transaction MUST include `signature` and `public_key`. Unsigned transactions are rejected with HTTP 200 `{"status":"error","msg":"Mainnet requires client-side signature."}`. The node will **never** sign on behalf of external addresses on mainnet.

#### Request Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `from` | string | Yes | Sender's LOS address (Base58Check) |
| `target` | string | Yes | Recipient's LOS address (Base58Check) |
| `amount` | u128 | No* | Amount in LOS (whole units). Use `amount_cil` for precision |
| `amount_cil` | u128 | No* | Amount in CIL (1 LOS = 100,000,000,000 CIL). Preferred for client-signed |
| `signature` | string | **Yes (mainnet)** | Hex-encoded Dilithium5 signature over `signing_hash` |
| `public_key` | string | **Yes (mainnet)** | Hex-encoded Dilithium5 public key of sender |
| `previous` | string | **Yes (mainnet)** | Hash of sender's latest block (get from `GET /bal/{address}` → `head` field) |
| `timestamp` | u64 | **Yes (mainnet)** | Unix timestamp in seconds (part of `signing_hash`) |
| `fee` | u128 | **Yes (mainnet)** | Fee in CIL. Must be ≥ `base_fee_cil` from `GET /fee-estimate/{address}` |
| `work` | u64 | No | PoW nonce. If omitted, the node computes it |

*Either `amount` or `amount_cil` must be provided. `amount_cil` is preferred for precision.

#### Client-Signed Transaction (Mainnet — Required)

The client must compute the `signing_hash`, sign it with Dilithium5, and include the signature.

**Step 1: Get account state and fee**

```
GET /bal/{sender_address}
→ { "head": "abc123...", "balance_cil": 5000000000000, ... }

GET /fee-estimate/{sender_address}
→ { "base_fee_cil": 100000, ... }

GET /node-info
→ { "protocol": { "chain_id_numeric": 1, "base_fee_cil": 100000, ... }, ... }
```

**Step 2: Build the `signing_hash`**

The `signing_hash` is a SHA3-256 hash of the following fields concatenated as raw bytes, in this exact order:

| # | Field | Encoding | Size |
|---|---|---|---|
| 1 | `chain_id` | u64 little-endian | 8 bytes |
| 2 | `account` | UTF-8 string bytes | variable |
| 3 | `previous` | UTF-8 string bytes | variable |
| 4 | `block_type` | single byte: Send=0, Receive=1, Change=2, Mint=3, Slash=4, ContractDeploy=5, ContractCall=6 | 1 byte |
| 5 | `amount` | u128 little-endian (in CIL) | 16 bytes |
| 6 | `link` | UTF-8 string bytes (= target address for Send) | variable |
| 7 | `public_key` | UTF-8 string bytes (hex-encoded public key) | variable |
| 8 | `work` | u64 little-endian (PoW nonce) | 8 bytes |
| 9 | `timestamp` | u64 little-endian | 8 bytes |
| 10 | `fee` | u128 little-endian (in CIL) | 16 bytes |

```
signing_hash = hex(SHA3-256(chain_id ‖ account ‖ previous ‖ block_type ‖ amount ‖ link ‖ public_key ‖ work ‖ timestamp ‖ fee))
```

**Important notes:**
- `chain_id`: Mainnet = `1`, Testnet = `2` (get from `GET /node-info` → `protocol.chain_id_numeric`)
- `amount`: Must be in CIL (u128 little-endian, 16 bytes) — same value as `amount_cil` in the request
- `link`: For Send blocks, this is the recipient address string
- `public_key`: The **hex-encoded** public key string (NOT raw bytes)
- `work`: If you pre-compute PoW, include the nonce. Otherwise use `0` and let the server compute it (set `work` field in request to omit)
- The **signature is NOT part of** `signing_hash` — it's computed over the hash

**Step 3: Compute PoW (optional)**

The block must satisfy Proof-of-Work: the `signing_hash` must have a minimum number of leading zero bits. If you omit `work` from the request, the node will compute it. If you compute it yourself, iterate nonces in the `work` field position until `signing_hash` meets the difficulty.

**Step 4: Sign with Dilithium5**

```
signature = hex(dilithium5_sign(signing_hash_bytes, secret_key))
```

Where `signing_hash_bytes` is the UTF-8 bytes of the hex-encoded signing_hash string (NOT the raw 32-byte hash).

**Step 5: Send the request**

```json
{
  "from": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "target": "LOSWoNusVctuR9TJKtpWa8fZdisdWk3XgznML",
  "amount_cil": 1000000000000,
  "signature": "a1b2c3...hex_dilithium5_signature_9254_bytes...",
  "public_key": "d4e5f6...hex_dilithium5_public_key_5184_chars...",
  "previous": "abc123def456...previous_block_hash...",
  "timestamp": 1771277598,
  "fee": 100000
}
```

#### Pseudocode Example (any language)

```python
import hashlib, struct, time

# 1. Get sender state
bal = GET("/bal/LOSMyAddress...")
node = GET("/node-info")
fee = GET("/fee-estimate/LOSMyAddress...")

chain_id = node["protocol"]["chain_id_numeric"]  # 1 for mainnet
previous = bal["head"]
amount_cil = 10 * 100_000_000_000  # 10 LOS in CIL
target = "LOSRecipientAddress..."
public_key_hex = "your_hex_encoded_public_key..."
timestamp = int(time.time())
base_fee_cil = fee["base_fee_cil"]  # 100000

# 2. Build signing_hash buffer
buf = b""
buf += struct.pack("<Q", chain_id)          # u64 LE
buf += "LOSMyAddress...".encode("utf-8")    # account
buf += previous.encode("utf-8")             # previous
buf += bytes([0])                           # block_type: Send = 0
buf += amount_cil.to_bytes(16, "little")    # u128 LE
buf += target.encode("utf-8")              # link
buf += public_key_hex.encode("utf-8")      # public_key (hex string!)
buf += struct.pack("<Q", 0)                # work (0 = let server compute)
buf += struct.pack("<Q", timestamp)        # timestamp u64 LE
buf += base_fee_cil.to_bytes(16, "little") # fee u128 LE

signing_hash = hashlib.sha3_256(buf).hexdigest()

# 3. Sign (the hex string bytes, not raw hash bytes!)
signature = dilithium5_sign(signing_hash.encode("utf-8"), secret_key)

# 4. Send
POST("/send", {
    "from": "LOSMyAddress...",
    "target": target,
    "amount_cil": amount_cil,
    "signature": signature.hex(),
    "public_key": public_key_hex,
    "previous": previous,
    "timestamp": timestamp,
    "fee": base_fee_cil,
})
```

#### Response (Success)

```json
{
  "status": "success",
  "tx_hash": "abc123def456...",
  "initial_power": 100,
  "fee_paid_cil": 100000,
  "fee_multiplier_bps": 10000
}
```

#### Error Responses

| Error | Description |
|---|---|
| `"Mainnet requires client-side signature"` | Missing `signature` + `public_key` on mainnet |
| `"Invalid signature: verification failed"` | Signature doesn't match signing_hash + public_key |
| `"public_key field is REQUIRED when providing signature"` | `signature` provided but `public_key` missing |
| `"Client fee X CIL is below minimum required fee Y CIL"` | Fee too low |
| `"Insufficient balance"` | Not enough balance for amount + fee + pending transactions |
| `"Invalid sender address format"` | `from` is not valid Base58Check |
| `"Invalid target address format"` | `target` is not valid Base58Check |
| `"Amount must be greater than 0"` | Zero-amount transaction rejected |
| `"Cannot send to your own address"` | Self-send rejected |
| `"Rate limit exceeded"` | Max 10 transactions per minute per address |
| `"Amount overflow: value too large"` | Amount × CIL_PER_LOS overflows u128 |
| `"Sender account not found"` | Sender address has no account on chain |

#### Node-Signed Transaction (Testnet Only)

> **This mode is DISABLED on mainnet.** Only available on testnet builds for development convenience.

For testnet, minimal fields are accepted and the node signs with its own key:

```json
{
  "target": "LOSWoNusVctuR9TJKtpWa8fZdisdWk3XgznML",
  "amount": 10
}
```

### GET `/transaction/{hash}`

Look up a transaction by its hash.

**Example:** `GET /transaction/abc123def456...`

### GET `/search/{query}`

Search across blocks, accounts, and transaction hashes.

**Example:** `GET /search/LOSX7dSt`

---

## Validator Endpoints

### GET `/validators`

List all active validators.

**Response:**
```json
{
  "validators": [
    {
      "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
      "active": true,
      "connected": true,
      "has_min_stake": true,
      "is_genesis": true,
      "onion_address": "f3zfmh...nid.onion",
      "stake": 1000,
      "uptime_percentage": 99
    }
  ]
}
```

### POST `/register-validator`

Register as a network validator. Requires Dilithium5 signature and ≥1 LOS balance. Reward eligibility requires ≥1,000 LOS.

**Request:**
```json
{
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "public_key": "hex_dilithium5_public_key...",
  "signature": "hex_dilithium5_signature...",
  "endpoint": "your-onion-address.onion:3030"
}
```

### POST `/unregister-validator`

Remove yourself from the validator set.

**Request:**
```json
{
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "public_key": "hex_dilithium5_public_key...",
  "signature": "hex_dilithium5_signature..."
}
```

---

## Consensus

### GET `/consensus`

aBFT consensus engine status and safety parameters.

**Response:**
```json
{
  "safety": {
    "active_validators": 4,
    "byzantine_threshold": 1,
    "byzantine_safe": true,
    "consensus_model": "aBFT"
  },
  "round": {
    "current": 128,
    "decided": 127
  }
}
```

### GET `/reward-info`

Validator reward pool and epoch information.

**Response:**
```json
{
  "epoch": {
    "current_epoch": 5,
    "epoch_reward_rate_los": 5000
  },
  "pool": {
    "remaining_los": 475000,
    "total_distributed_los": 25000
  },
  "validators": {
    "eligible": 4,
    "total": 4
  }
}
```

### GET `/slashing`

Global slashing statistics.

### GET `/slashing/{address}`

Slashing profile for a specific validator address.

---

## Smart Contract Endpoints

### POST `/deploy-contract`

Deploy a WASM smart contract to the UVM.

**Request:**
```json
{
  "wasm_hex": "0061736d...",
  "deployer": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
  "signature": "hex_signature...",
  "public_key": "hex_public_key..."
}
```

### POST `/call-contract`

Execute a function on a deployed smart contract.

**Request:**
```json
{
  "contract_id": "contract_address_or_hash",
  "function": "transfer",
  "args": ["LOSX7dSt...", "1000"],
  "caller": "LOSX7dSt...",
  "signature": "hex_signature...",
  "public_key": "hex_public_key..."
}
```

### GET `/contract/{id}`

Get the state and info of a deployed contract.

### GET `/contracts`

List all deployed contracts.

---

## Network Endpoints

### GET `/peers`

Connected peers and validator endpoints.

**Response:**
```json
{
  "peer_count": 5,
  "peers": [
    {
      "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
      "host_address": "kljkjq...kyad.onion:3030",
      "onion_address": "kljkjq...kyad.onion:3030",
      "is_validator": true,
      "self": true,
      "short_address": "los_X7dStdPk"
    }
  ],
  "validator_endpoint_count": 5,
  "validator_endpoints": [
    {
      "address": "LOSX7dSt...",
      "host_address": "kljkjq...kyad.onion:3030",
      "onion_address": "kljkjq...kyad.onion:3030"
    }
  ]
}
```

> **Port in host_address:** The `host_address` and `onion_address` fields include the REST port suffix (e.g. `abc.onion:3030`). This is the actual port the validator's REST API listens on. Tor's `HiddenServicePort` maps the `.onion` virtual port to the local port — port 80 is **never** used.

### GET `/network/peers`

Network-level peer discovery with endpoint information. Designed for Flutter app peer discovery. Each entry includes `transport` ("onion" or "clearnet"), `rest_port` (actual REST API port), and `stake_los`.

**Response:**
```json
{
  "version": 1,
  "total": 5,
  "timestamp": 1740500000,
  "endpoints": [
    {
      "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
      "host_address": "kljkjq...kyad.onion:3030",
      "onion_address": "kljkjq...kyad.onion:3030",
      "transport": "onion",
      "rest_port": 3030,
      "stake_los": 1000,
      "reachable": true
    }
  ]
}
```

> **Note:** `rest_port` is extracted from the `host_address` port suffix. If the host has no port suffix, it defaults to `80`. The `transport` field helps Flutter apps determine whether to use a SOCKS5 proxy (for `.onion`) or direct HTTP (for clearnet).

### GET `/directory/api/peers`

All known peers as JSON — used by the embedded Peer Directory.

**Response:**
```json
{
  "network": "mainnet",
  "active_count": 5,
  "total_count": 5,
  "updated_at": "2026-02-25T12:00:00Z",
  "peers": [
    {
      "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
      "host": "http://kljkjq...kyad.onion:3030",
      "transport": "onion",
      "active": true,
      "stake_los": 1000,
      "rest_port": 3030,
      "is_bootstrap": true
    }
  ]
}
```

### GET `/directory/api/active`

Active (reachable) peers only — optimized for app bootstrapping.

**Response:**
```json
{
  "network": "mainnet",
  "active_count": 5,
  "updated_at": "2026-02-25T12:00:00Z",
  "peers": [
    {
      "host": "http://kljkjq...kyad.onion:3030",
      "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1",
      "transport": "onion",
      "rest_port": 3030
    }
  ]
}
```

### GET `/directory`

HTML Peer Directory page — a human-readable dashboard showing all known validators, their `.onion` addresses, active/inactive status, and stake amounts. Accessible via browser.

### GET `/mempool/stats`

Current mempool statistics.

**Response:**
```json
{
  "pending_transactions": 0,
  "queued": 0
}
```

### GET `/sync`

GZIP-compressed ledger state for node synchronization. Use `?from={block_count}` for incremental sync.

### GET `/whoami`

This node's signing address.

**Response:**
```json
{
  "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1"
}
```

---

## Wallet Endpoints

### POST `/create-wallet`

Create a new Dilithium5 (post-quantum) wallet. The private key is encrypted with the provided password before being returned.

**Request:**
```json
{
  "password": "your_secure_password_12chars"
}
```

**Response:**
```json
{
  "status": "success",
  "address": "LOSHjvLcaLZpKcRvHoEKtYdbQbMZECzNp3gh9LJ7Y9ZPTqH",
  "public_key": "a1b2c3d4...hex_encoded_dilithium5_public_key",
  "encrypted_secret_key": "base64_encoded_encrypted_private_key...",
  "encryption_version": 1,
  "note": "Store encrypted_secret_key safely. You need your password to decrypt it for signing transactions."
}
```

**Errors:**
- `400` — Password too short (minimum 12 characters)
- `400` — Invalid request body

**Security Notes:**
- The raw secret key is never transmitted — it is encrypted with `age` (scrypt-based) before being returned.
- Store `encrypted_secret_key` securely. Without your password, the key cannot be recovered.
- The `public_key` and `address` are safe to share publicly.

**Transaction Flow After Wallet Creation:**
1. Call `POST /create-wallet` → save `address`, `public_key`, `encrypted_secret_key`
2. To send LOS: decrypt the secret key locally with your password
3. Follow the [complete signing guide in POST /send](#post-send) to build `signing_hash`, sign, and submit

---

## Utility Endpoints

### GET `/tor-health`

Tor hidden service self-check status.

**Response:**
```json
{
  "onion_reachable": true,
  "consecutive_failures": 0,
  "total_pings": 100,
  "total_failures": 2
}
```

### POST `/faucet`

Claim testnet tokens (disabled on mainnet).

**Request:**
```json
{ "address": "LOSX7dStdPkS9U4MFCmDQfpmvrbMa5WAZfQX1" }
```

---

## gRPC API

Protocol definition: [`los.proto`](../los.proto)

| RPC Method | Description |
|---|---|
| `GetBalance` | Account balance |
| `GetAccount` | Full account details |
| `GetBlock` | Block by hash |
| `GetLatestBlock` | Latest block |
| `SendTransaction` | Submit signed transaction |
| `GetNodeInfo` | Node information |
| `GetValidators` | Validator list |
| `GetBlockHeight` | Current block height |

**gRPC port:** Always REST port + 20,000 (default: `23030`).

---

## USP-01 Token Endpoints

The **USP-01 Token Standard** is deployed as WASM contracts on the UVM. These operations go through the generic `/deploy-contract` and `/call-contract` endpoints, but with specific function signatures documented here.

### Deploy a USP-01 Token

Use `POST /deploy-contract` with a compiled USP-01 WASM binary, then call `init`.

**Init Call:**
```json
{
  "contract_id": "LOSConXXXX...",
  "function": "init",
  "args": ["My Token", "MTK", "11", "1000000", "0", "", "0", ""],
  "caller": "LOSX7dSt...",
  "signature": "hex...",
  "public_key": "hex..."
}
```

| Arg | Field | Type | Description |
|---|---|---|---|
| 0 | `name` | String | Token name (1-64 chars) |
| 1 | `symbol` | String | Token symbol (1-8 chars) |
| 2 | `decimals` | u8 | Decimal places (0-18) |
| 3 | `total_supply` | u128 string | Initial supply assigned to deployer |
| 4 | `is_wrapped` | "0"/"1" | Whether this is a wrapped asset |
| 5 | `wrapped_origin` | String | Source chain identifier (e.g. "ETH") |
| 6 | `max_supply` | u128 string | Max supply cap ("0" = no cap) |
| 7 | `bridge_operator` | address | Address authorized for wrap_mint |

### `transfer`

Transfer tokens from caller to recipient.

```json
{ "function": "transfer", "args": ["LOSRecipient...", "1000"] }
```

### `approve`

Set spending allowance for a spender. Set amount to "0" to revoke.

```json
{ "function": "approve", "args": ["LOSSpender...", "5000"] }
```

### `transfer_from`

Transfer tokens using a pre-approved allowance.

```json
{ "function": "transfer_from", "args": ["LOSOwner...", "LOSRecipient...", "1000"] }
```

### `burn`

Permanently destroy tokens from caller's balance. Reduces total supply.

```json
{ "function": "burn", "args": ["500"] }
```

### `balance_of` (Read-only)

```json
{ "function": "balance_of", "args": ["LOSHolder..."] }
```

**Response:**
```json
{ "account": "LOSHolder...", "balance": "1000" }
```

### `allowance_of` (Read-only)

```json
{ "function": "allowance_of", "args": ["LOSOwner...", "LOSSpender..."] }
```

**Response:**
```json
{ "owner": "LOSOwner...", "spender": "LOSSpender...", "allowance": "5000" }
```

### `total_supply` (Read-only)

```json
{ "function": "total_supply", "args": [] }
```

**Response:**
```json
{ "total_supply": "1000000" }
```

### `token_info` (Read-only)

Returns full token metadata.

**Response:**
```json
{
  "name": "My Token",
  "symbol": "MTK",
  "decimals": 11,
  "total_supply": "1000000",
  "is_wrapped": false,
  "wrapped_origin": "",
  "max_supply": "0",
  "bridge_operator": "",
  "owner": "LOSX7dSt...",
  "contract": "LOSConXXXX...",
  "standard": "USP-01"
}
```

### `wrap_mint` (Bridge Operator Only)

Mint wrapped tokens upon cross-chain deposit verification.

```json
{ "function": "wrap_mint", "args": ["LOSRecipient...", "1000", "0xTxProof..."] }
```

### `wrap_burn`

Burn wrapped tokens for redemption on the source chain.

```json
{ "function": "wrap_burn", "args": ["500", "0xDestinationAddress..."] }
```

**Events emitted:** `USP01:Init`, `USP01:Transfer`, `USP01:Approval`, `USP01:Burn`, `USP01:WrapMint`, `USP01:WrapBurn`.

---

## DEX AMM Endpoints

The **DEX AMM** is a constant-product (x·y=k) automated market maker deployed as a WASM contract. All operations go through `/deploy-contract` and `/call-contract`.

**Constants:** 0.3% fee (30 bps), minimum liquidity 1,000, max fee 1,000 bps.

### `init`

Initialize the DEX contract.

```json
{ "function": "init", "args": [] }
```

### `create_pool`

Create a new liquidity pool with initial reserves.

```json
{
  "function": "create_pool",
  "args": ["LOSConTokenA...", "LOSConTokenB...", "1000000", "500000", "30"]
}
```

| Arg | Field | Type | Description |
|---|---|---|---|
| 0 | `token_a` | address/"LOS" | First token (use "LOS" for native) |
| 1 | `token_b` | address/"LOS" | Second token |
| 2 | `amount_a` | u128 string | Initial reserve for token A |
| 3 | `amount_b` | u128 string | Initial reserve for token B |
| 4 | `fee_bps` | u128 string | Fee in basis points (optional, default 30) |

**LP minted:** `isqrt(amount_a × amount_b) - 1000` (minimum liquidity locked).

### `add_liquidity`

Add proportional liquidity to an existing pool.

```json
{ "function": "add_liquidity", "args": ["0", "100000", "50000", "900"] }
```

| Arg | Field | Description |
|---|---|---|
| 0 | `pool_id` | Pool identifier |
| 1 | `amount_a` | Token A deposit |
| 2 | `amount_b` | Token B deposit |
| 3 | `min_lp_tokens` | Slippage protection: minimum LP tokens to accept |

### `remove_liquidity`

Withdraw proportional reserves by burning LP tokens.

```json
{ "function": "remove_liquidity", "args": ["0", "500", "40000", "20000"] }
```

| Arg | Field | Description |
|---|---|---|
| 0 | `pool_id` | Pool identifier |
| 1 | `lp_amount` | LP tokens to burn |
| 2 | `min_amount_a` | Slippage protection: minimum token A out |
| 3 | `min_amount_b` | Slippage protection: minimum token B out |

### `swap`

Execute a token swap with MEV protection.

```json
{ "function": "swap", "args": ["0", "LOSConTokenA...", "10000", "4800", "1771280000"] }
```

| Arg | Field | Description |
|---|---|---|
| 0 | `pool_id` | Pool identifier |
| 1 | `token_in` | Address of token being sold |
| 2 | `amount_in` | Amount to swap |
| 3 | `min_amount_out` | Slippage protection: minimum output |
| 4 | `deadline` | Unix timestamp deadline (MEV protection) |

**Formula:** `amount_out = (amount_after_fee × reserve_out) / (reserve_in + amount_after_fee)`

### `get_pool` (Read-only)

```json
{ "function": "get_pool", "args": ["0"] }
```

**Response:**
```json
{
  "pool_id": "0",
  "token_a": "LOSConTokenA...",
  "token_b": "LOSConTokenB...",
  "reserve_a": "1000000",
  "reserve_b": "500000",
  "total_lp": "706106",
  "fee_bps": "30",
  "creator": "LOSX7dSt...",
  "last_trade": "1771277598",
  "spot_price_scaled": "2000000000000"
}
```

### `quote` (Read-only)

Get expected swap output without executing.

```json
{ "function": "quote", "args": ["0", "LOSConTokenA...", "10000"] }
```

**Response:**
```json
{
  "amount_out": "4950",
  "fee": "30",
  "price_impact_bps": "100",
  "spot_price_scaled": "2000000000000"
}
```

### `get_position` (Read-only)

Get caller's LP position in a pool.

```json
{ "function": "get_position", "args": ["0"] }
```

**Response:**
```json
{
  "lp_shares": "10000",
  "total_lp": "706106",
  "amount_a": "14158",
  "amount_b": "7079",
  "share_pct_bps": "141"
}
```

### `list_pools` (Read-only)

List all pools in the DEX.

```json
{ "function": "list_pools", "args": [] }
```

**Events emitted:** `DexInit`, `PoolCreated`, `LiquidityAdded`, `LiquidityRemoved`, `Swap`.

---

## CLI Reference

The `los-cli` binary provides command-line access to all node functionality.

**Global flags:** `--rpc <URL>` (default: `http://localhost:3030`), `--config-dir <DIR>` (default: `~/.los`)

### `los-cli wallet` — Wallet Management

| Command | Description |
|---|---|
| `wallet new --name <NAME>` | Create new Dilithium5 wallet |
| `wallet list` | List all wallets |
| `wallet balance <ADDRESS>` | Show wallet balance |
| `wallet export <NAME> --output <PATH>` | Export encrypted wallet |
| `wallet import <PATH> --name <NAME>` | Import wallet |

### `los-cli tx` — Transaction Operations

| Command | Description |
|---|---|
| `tx send --to <ADDR> --amount <LOS> --from <WALLET>` | Send LOS to address |
| `tx status <HASH>` | Query transaction status |

### `los-cli query` — Blockchain Queries

| Command | Description |
|---|---|
| `query block <HEIGHT>` | Get block by height |
| `query account <ADDRESS>` | Get account state |
| `query info` | Network information |
| `query validators` | Get validator set |

### `los-cli validator` — Validator Operations

| Command | Description |
|---|---|
| `validator stake --amount <LOS> --wallet <NAME>` | Stake tokens (min 1,000 LOS) |
| `validator unstake --wallet <NAME>` | Unstake tokens |
| `validator status <ADDRESS>` | Show validator status |
| `validator list` | List active validators |

### `los-cli token` — USP-01 Token Operations

| Command | Description |
|---|---|
| `token deploy --wallet <W> --wasm <PATH> --name <N> --symbol <S> --decimals <D> --total-supply <AMT>` | Deploy USP-01 token |
| `token list` | List all deployed tokens |
| `token info <ADDRESS>` | Show token metadata |
| `token balance --token <ADDR> --holder <ADDR>` | Query token balance |
| `token allowance --token <T> --owner <O> --spender <S>` | Query allowance |
| `token transfer --wallet <W> --token <T> --to <ADDR> --amount <AMT>` | Transfer tokens |
| `token approve --wallet <W> --token <T> --spender <ADDR> --amount <AMT>` | Approve spender |
| `token burn --wallet <W> --token <T> --amount <AMT>` | Burn tokens |
| `token mint --wallet <W> --token <T> --to <ADDR> --amount <AMT>` | Distribute tokens (owner transfer) |

### `los-cli dex` — DEX Operations

| Command | Description |
|---|---|
| `dex deploy --wallet <W> --wasm <PATH>` | Deploy DEX AMM contract |
| `dex pools` | List all DEX pools |
| `dex pool --contract <C> --pool-id <ID>` | Show pool info |
| `dex quote --contract <C> --pool-id <ID> --token-in <T> --amount-in <AMT>` | Get swap quote |
| `dex position --contract <C> --pool-id <ID> --user <ADDR>` | Get LP position |
| `dex create-pool --wallet <W> --contract <C> --token-a <A> --token-b <B> --amount-a <A> --amount-b <B>` | Create liquidity pool |
| `dex add-liquidity --wallet <W> --contract <C> --pool-id <ID> --amount-a <A> --amount-b <B> --min-lp <MIN>` | Add liquidity |
| `dex remove-liquidity --wallet <W> --contract <C> --pool-id <ID> --lp-amount <LP> --min-a <A> --min-b <B>` | Remove liquidity |
| `dex swap --wallet <W> --contract <C> --pool-id <ID> --token-in <T> --amount-in <A> --min-out <MIN>` | Execute swap |

---

## Rate Limits

| Endpoint | Limit |
|---|---|
| `/faucet` | 1 per address per 24 hours |
| `/send` | Anti-spam throttle per address |
| All endpoints | Per-IP rate limiting |
