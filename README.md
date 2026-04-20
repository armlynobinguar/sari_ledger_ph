# SariLedger

On-chain inventory, sales ledger, and revenue-backed micro-loans for Filipino sari-sari stores.

## Problem

Aling Marites, who runs a sari-sari store in Cainta, Rizal earning ~₱25,000/month, tracks sales and "utang" in a pocket notebook — she loses ~20% of potential profit to forgotten credit and stockouts, and has zero documented revenue to qualify for the ₱50,000 BPI loan she needs to expand.

## Solution

SariLedger lets her log every restock and sale via a mobile app that writes to a Soroban contract, building verifiable on-chain revenue history that automatically unlocks USDC-denominated micro-loans from community lenders, settled instantly on Stellar for fractions of a centavo.

## Timeline

| Phase | Scope | Duration |
|---|---|---|
| Week 1 | Contract logic (inventory, sales ledger, loan eligibility) + tests | 5 days |
| Week 2 | PWA frontend (record sale / restock / request loan flows) | 5 days |
| Week 3 | Testnet integration, USDC trustline, offline queue | 3 days |
| Week 4 | Pilot with 3 sari-sari stores in Metro Manila, demo video | 3 days |

## Stellar Features Used

- **Soroban smart contracts** — core inventory, sales ledger, and loan eligibility logic
- **USDC transfers** — loan disbursement and repayment (via trustline)
- **Trustlines** — USDC on owner and lender accounts

## Vision and Purpose

The Philippine sari-sari economy generates ₱2.6T annually and accounts for 30% of retail GDP — yet 99% of its operators (82% of whom are women) remain unbanked because they have no documented revenue. SariLedger turns every sale into a cryptographically verifiable economic signal, creating the first open credit primitive for Southeast Asia's informal retail layer. Once revenue history is composable on-chain, any DeFi lender, insurer, or cooperative can underwrite these micro-entrepreneurs at a cost traditional banks cannot match.

## Prerequisites

- Rust 1.81+ with the `wasm32v1-none` target:
  ```bash
  rustup target add wasm32v1-none
  ```
- Soroban CLI (a.k.a. `stellar` CLI) v22.0+:
  ```bash
  cargo install --locked stellar-cli
  ```
- A funded testnet identity:
  ```bash
  stellar keys generate --global aling-marites --network testnet --fund
  ```

## Build

```bash
stellar contract build
```

This produces `target/wasm32v1-none/release/sari_ledger.wasm`.

## Test

```bash
cargo test
```

All 5 tests should pass (happy path sale, insufficient inventory, multi-sale state, loan within limit, loan over limit).

## Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/sari_ledger.wasm \
  --source aling-marites \
  --network testnet \
  --alias sari_ledger
```

Initialize the contract with the store owner's address:

```bash
stellar contract invoke \
  --id sari_ledger \
  --source aling-marites \
  --network testnet \
  -- initialize \
  --owner $(stellar keys address aling-marites)
```

## Sample CLI Invocation (MVP)

Record a restock of 50 units of rice at ₱45 per unit:

```bash
stellar contract invoke \
  --id sari_ledger \
  --source aling-marites \
  --network testnet \
  -- restock \
  --product_id RICE \
  --quantity 50 \
  --cost 45
```

Record a sale of 10 units of rice at ₱55 each:

```bash
stellar contract invoke \
  --id sari_ledger \
  --source aling-marites \
  --network testnet \
  -- record_sale \
  --product_id RICE \
  --quantity 10 \
  --price 55
```

Request a ₱150 loan against documented revenue:

```bash
stellar contract invoke \
  --id sari_ledger \
  --source aling-marites \
  --network testnet \
  -- request_loan \
  --amount 150
```

Check revenue:

```bash
stellar contract invoke \
  --id sari_ledger \
  --source aling-marites \
  --network testnet \
  -- get_revenue
```
## Deployment

**Network:** Stellar Testnet (Test SDF Network ; September 2015)
**Contract ID:** `CCXZOPW4XW3KPJG2MRGAGW2HUTSYQCLD3FGCM2TMQSVJ5QFTABELGPZY`
**Deployed:** April 20, 2026

### Explore the deployed contract

- [View on Stellar Expert](https://stellar.expert/explorer/testnet/contract/CCXZOPW4XW3KPJG2MRGAGW2HUTSYQCLD3FGCM2TMQSVJ5QFTABELGPZY)
- [Open in Stellar Lab](https://lab.stellar.org/r/testnet/contract/CCXZOPW4XW3KPJG2MRGAGW2HUTSYQCLD3FGCM2TMQSVJ5QFTABELGPZY)
- [Deployment transaction](https://stellar.expert/explorer/testnet/tx/2b17016e3b9257251bc808d4a38146c3d7dfc9490603c54311d3a48af19329bc)

### Interact with the live contract

Initialize (one-time, sets the store owner):

```bash
stellar contract invoke \
  --id CCXZOPW4XW3KPJG2MRGAGW2HUTSYQCLD3FGCM2TMQSVJ5QFTABELGPZY \
  --source alice \
  --network testnet \
  -- initialize \
  --owner $(stellar keys address alice)
```

Record a restock of 50 units of rice at ₱45 per unit:

```bash
stellar contract invoke \
  --id CCXZOPW4XW3KPJG2MRGAGW2HUTSYQCLD3FGCM2TMQSVJ5QFTABELGPZY \
  --source alice \
  --network testnet \
  -- restock \
  --product_id RICE \
  --quantity 50 \
  --cost 45
```

Record a sale of 10 units at ₱55:

```bash
stellar contract invoke \
  --id CCXZOPW4XW3KPJG2MRGAGW2HUTSYQCLD3FGCM2TMQSVJ5QFTABELGPZY \
  --source alice \
  --network testnet \
  -- record_sale \
  --product_id RICE \
  --quantity 10 \
  --price 55
```

Check documented revenue:

```bash
stellar contract invoke \
  --id CCXZOPW4XW3KPJG2MRGAGW2HUTSYQCLD3FGCM2TMQSVJ5QFTABELGPZY \
  --source alice \
  --network testnet \
  -- get_revenue
```

## License

MIT © 2026 SariLedger contributors
