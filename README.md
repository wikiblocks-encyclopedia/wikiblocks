# WikiBlocks

WikiBlocks is a censorship-resistant free encyclopedia powered by blockchain technology. Built by using Polkadot Substrate framework. This is repository contains the node code for the WikiBlocks chain.

## Getting Started

### Install rustup

##### Linux

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

##### macOS

```
brew install rustup
```

### Install Rust

```
rustup update
rustup toolchain install stable
rustup target add wasm32-unknown-unknown
rustup toolchain install nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```

### Clone and Build WikiBlocks

```
git clone https://github.com/https://github.com/wikiblocks-encyclopedia/wikiblocks
cd wikiblocks
cargo build --release --all-features
```
