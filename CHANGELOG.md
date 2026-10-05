# Changelog

All notable changes to the Celor project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-10-05
### Added
- **Initial Public Release of Celor Core.**
- Full implementation of Alpenglow protocol detection and SIMD-0525 slot boundaries.
- Cross-provider reconciliation for public JSON-RPC and Yellowstone gRPC.
- Validator-local `bank_id` tracking and BankGraph implementation.
- `UpdateParent` fast leader handover telemetry pipeline.
- BLS Certificate Notarization engine (SIMD-0326) with decode and verification functionality.
- TowerBFT 32-lockout legacy fallback modes.
- `celor` CLI with commands: `serve`, `status`, `live`, `inspect`, `coverage`, `capture`, `replay`.
- High-performance execution routing interface.
- Complete OpenAPI and WebSocket streams for self-hosting.
- Next.js Demonstration UI (`celor-ui`) with Live Telemetry, BankGraph, and Transaction Autopsy sections.
