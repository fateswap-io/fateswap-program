# FateSwap Program

Solana on-chain program for [FateSwap](https://fateswap.io) — a provably fair prediction market where you trade at the price you believe in.

## Program ID

```
EHYqhQQLKRy1Don3p57B3FozPM8TVHKip6tsSx9Nhp4k
```

[View on Solana Explorer](https://explorer.solana.com/address/EHYqhQQLKRy1Don3p57B3FozPM8TVHKip6tsSx9Nhp4k)

## Overview

FateSwap is a DEX-framed prediction market built on Solana. Users place "fate orders" at a chosen multiplier (1.01x–10x), and outcomes are determined using a provably fair system where the server seed is committed on-chain before the trade is placed.

### Instructions

| Instruction | Description |
|---|---|
| `initialize` | Initialize the ClearingHouse (LP pool) |
| `deposit_sol` | Deposit SOL and mint fSOL LP tokens |
| `withdraw_sol` | Burn fSOL LP tokens and withdraw SOL |
| `submit_commitment` | Commit server seed hash on-chain (before trade) |
| `place_fate_order` | Place a fate order at a chosen multiplier |
| `settle_fate_order` | Settle an order and distribute funds |
| `reclaim_expired_order` | Reclaim funds from an expired order |
| `set_referrer` | Set a referral relationship (one-time) |
| `pause` | Pause/unpause the protocol |
| `update_config` | Update protocol configuration |
| `update_settler` | Update the settler wallet |
| `create_lp_metadata` | Create Metaplex metadata for the fSOL LP token |

### Provably Fair

1. Server generates a random seed and commits `SHA256(seed)` on-chain via `submit_commitment`
2. User places their trade via `place_fate_order`
3. Outcome is derived from `SHA256(server_seed || wallet_address || nonce)` — deterministic and verifiable
4. Server seed is revealed at settlement, and anyone can verify the outcome

The commitment is locked on-chain before the user trades, so the server cannot manipulate results.

## Build

### Prerequisites

- Rust 1.79.0 (pinned via `rust-toolchain.toml`)
- Solana CLI 1.18.26
- Anchor CLI 0.30.1

### Build the program

```bash
anchor build --no-idl -p fateswap
```

The compiled binary will be at `target/deploy/fateswap.so`.

## Verify

You can verify that the deployed program matches this source code using [solana-verify](https://github.com/Ellipsis-Labs/solana-verifiable-build):

```bash
# Install solana-verify
cargo install solana-verify

# Verify against this repo
solana-verify verify-from-repo \
  --program-id EHYqhQQLKRy1Don3p57B3FozPM8TVHKip6tsSx9Nhp4k \
  https://github.com/fateswap-io/fateswap-program \
  --mount-path programs/fateswap
```

## Security

If you discover a vulnerability, please report it responsibly via info@fateswap.io.

## License

MIT — see [LICENSE](LICENSE).
