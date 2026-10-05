# Contributing to Celor

Thank you for your interest in contributing to Celor! We welcome contributions from the community to help build the standard infrastructure for the Solana Alpenglow era.

## How to Contribute

1. **Fork the repository** on GitHub.
2. **Create a new branch** for your feature or bugfix (`git checkout -b feature/my-new-feature`).
3. **Commit your changes** with descriptive commit messages.
4. **Run the tests** (`cargo test --workspace`) and ensure `cargo clippy` passes.
5. **Push to your branch** (`git push origin feature/my-new-feature`).
6. **Open a Pull Request** against the `main` branch.

## Code Style

- We follow standard Rust formatting. Run `cargo fmt` before committing.
- Ensure all tests pass.
- Write unit tests for new functionality, especially core consensus logic.

## Developing Locally

See the [README.md](README.md) for quickstart instructions on running the local geyser fixture or connecting to a live cluster.
