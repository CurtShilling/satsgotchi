# Satsgotchi

Bitcoin's first digital pet game - combining Ordinals NFTs with Arch Network smart contracts.

## What is Satsgotchi?

A Tamagotchi-style virtual pet game where:
- Your pet is a Bitcoin Ordinal NFT (permanent, immutable)
- Game logic runs on Arch Network (fast, cheap)
- $GOTCHI token powers the ecosystem (deflationary)
- 99.5% of pets die - only 0.05% achieve Ascension

## Repository Structure

```
satsgotchi/
├── contracts/          # Arch Network smart contracts
│   ├── satsgotchi/    # Game state program
│   ├── token/         # $GOTCHI token program
│   └── oracle/        # Bitcoin ↔ Arch sync service
├── frontend/          # User interface
├── scripts/           # Testing & deployment
└── docs/              # Documentation
```

## Quick Start

### Prerequisites
- Rust (latest stable)
- Arch CLI tools
- Bitcoin testnet node (for oracle)

### Build Contracts
```bash
cd contracts/satsgotchi
cargo build-bpf

cd ../token
cargo build-bpf

cd ../oracle
cargo build --release
```

### Run Tests
```bash
./scripts/test.sh
```

### Deploy (when Arch launches)
```bash
arch-cli program deploy contracts/satsgotchi/target/deploy/satsgotchi.so
arch-cli program deploy contracts/token/target/deploy/gotchi_token.so
```

## Game Mechanics

### Evolution Stages
- Baby (7 days) → 50 $GOTCHI
- Child (28 days) → 250 $GOTCHI
- Teen (120 days) → 1,500 $GOTCHI
- Adult (120 days) → 25,000 $GOTCHI
- Senior (80 days) → Eligible for Ascension
- Ascended → 2,000,000 $GOTCHI

### Care Actions
- Feed (burns $GOTCHI)
- Play (burns $GOTCHI)
- Clean (burns $GOTCHI)
- Medicine (burns $GOTCHI)

### Death Mechanics
- 99.5% death rate over full lifecycle
- Neglect increases death probability
- Dead pets become memorial NFTs

## Tokenomics

**$GOTCHI Supply:**
- Max: 1,000,000,000 (1B)
- Milestone Pool: 300M (evolution rewards)
- Earning Pool: 300M (passive income)
- Liquidity: 150M
- Team/Marketing: 250M

**Fees:**
- Buy: 0.5%
- Sell: 0.75%
- Weekly buyback & burn

## Status

- [x] Smart contracts written
- [x] Frontend created
- [x] Oracle service built
- [ ] Local testing
- [ ] Testnet deployment
- [ ] Security audit
- [ ] Mainnet launch (when Arch launches)

## License

Proprietary - All Rights Reserved

## Contact

- Twitter: [@Satsgotchi](https://twitter.com/satsgotchi)
- Discord: Coming soon
- Website: https://satsgotchi.com (coming soon)
