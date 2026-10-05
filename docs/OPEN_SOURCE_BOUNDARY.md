# CELOR Open-Source Boundary

The Celor architecture is designed with a clean separation between the Open-Source Software (OSS) core and the future managed Celor Cloud service. 

Our strategy is: **Open the Engine. Charge for the Ride.**

Developers should be able to download, run, modify, and self-host Celor for free. Celor Cloud will provide the easiest, fastest, and most reliable managed way to scale it.

## Feature Matrix

| FEATURE | OPEN SOURCE? | SELF-HOSTABLE? | CELOR CLOUD? | FUTURE PAID? |
| --- | --- | --- | --- | --- |
| **Core Protocol Engine** | Yes | Yes | Yes | No |
| **RPC / WS Adapters** | Yes | Yes | Yes | No |
| **Yellowstone Adapter** | Yes | Yes | Yes | No |
| **Geyser Plugin** | Yes | Yes | Yes | No |
| **Validator Telemetry** | Yes | Yes | Yes | No |
| **Bank Graph / Parent Tracking** | Yes | Yes | Yes | No |
| **Certificates & BLS Engine** | Yes | Yes | Yes | No |
| **Multi-Provider Reconciliation** | Yes | Yes | Yes | No |
| **Replay & Capture CLI** | Yes | Yes | Yes | No |
| **Self-Hosted API / WebSocket** | Yes | Yes | Yes | No |
| **SDK (TS / Rust)** | Yes | Yes | Yes | No |
| **Demo UI** | Yes | Yes | Yes | No |
| **QUIC Reference Route** | Yes | Yes | Yes | No |
| **Managed Low-Latency Routing** | No | No | Yes | Yes |
| **Hosted API & WebSocket** | No | No | Yes | Yes |
| **Historical Data & Long Retention** | No | No | Yes | Yes |
| **Global Aggregation** | No | No | Yes | Yes |
| **Billing & Auth** | No | No | Yes | Yes |
| **Enterprise SLA & Support** | No | No | Yes | Yes |
| **Private Infrastructure** | No | No | Yes | Yes |

## What remains Private and Why

1. **Hosted Infrastructure and Scaling Code:** The orchestration, fleet management, and node configurations required to run Celor at global scale are proprietary. This forms the operational moat.
2. **Managed Execution & Premium Routing:** While the interface for execution exists in OSS, the dedicated private connectivity, premium endpoints, and enterprise QUIC infrastructure are commercial products.
3. **Historical Data Lakes:** Storing, indexing, and serving months or years of high-resolution validator telemetry and transaction histories at scale requires expensive storage architecture, which will be offered as a premium cloud data product.
4. **Billing, Auth & Enterprise Features:** SSO, RBAC, API key issuance, Stripe integration, and metering are specific to the SaaS business and do not belong in the open-source infrastructure project.

All core infrastructure required to understand Alpenglow consensus, track state, and run Celor locally is entirely open and Apache 2.0 licensed.
