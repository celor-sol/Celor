# Celor

Fast infrastructure for Solana's new Alpenglow consensus.

![Celor](https://github.com/celor-sol/celor/raw/main/public/celorbanner.png)

## What is this?

Solana is moving from legacy TowerBFT to Alpenglow (multi-candidate banks, fast leader handovers, BLS certificates). If you're building a bot, an indexer, or a trading terminal, the old RPC methods are going to be too slow and miss the micro-second reorgs. 

Celor normalizes all the new Alpenglow data (from Yellowstone gRPC, Geyser, or standard RPC) into one fast, clean stream.

```text
Solana -> Yellowstone/Geyser -> Celor -> Your App
```

It gives you:
- **Real-time slots & leaders**: Across any provider.
- **UpdateParent & Bank Graph**: Track exactly when a leader drops a block and hands over to the next one, instantly.
- **BLS Certificates**: See the actual 80% stake notarization certs.
- **Sub-150ms Finality**: Don't wait for 32 slots. Get the fast-path finality.

## Run it locally

Celor is open source and built to run on your own machine or server. 

### 1. Build

```bash
git clone git@github.com:celor-sol/celor.git
cd Celor
cargo build --workspace
```

### 2. Start the API

Run the server on devnet to see the data flow:

```bash
cargo run --bin celor -- serve --cluster devnet
```

### 3. Stream the data

Connect your app to `ws://localhost:8900/api/v1/stream`.

Want to see what's happening right now in the terminal? Run the inspector:

```bash
cargo run --bin celor -- inspect
```

## Docs & Architecture

Check the `/docs` folder for deep dives:
- [How Celor works under the hood](./docs/ARCHITECTURE.md)
- [Provider support matrix](./docs/PHASE_7_PROVIDER_MATRIX.md)
- [Open Source vs Cloud Boundary](./docs/OPEN_SOURCE_BOUNDARY.md)

## Open Source 

The core engine of Celor is free and Apache 2.0 licensed. 
We believe the core infrastructure should be open. We will charge later for hosted endpoints, massive historical data lakes, and managed routing (Celor Cloud), but the engine itself is yours to run.
