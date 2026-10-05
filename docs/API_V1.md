# CHRONO HTTP & WebSocket API (v1)

> **Specification Version**: 1.0.0  
> **Default Service Host**: `127.0.0.1`  
> **Default Service Port**: `8900`  
> **Protocol**: HTTP/1.1 & WebSocket (RFC 6455)  
> **Provenance Standard**: `DIRECT`, `DERIVED`, `ESTIMATED`, `UNAVAILABLE`

---

## 1. Overview & Architecture Boundary

The Chrono Service API represents the authoritative boundary between the Solana network and downstream consumer applications (such as the React UI, local trading bots, or telemetry collectors).

```
Solana / Yellowstone / Geyser / RPC
                ↓
       CHRONO RUST CORE
                ↓
    NORMALIZED CHRONO STATE
                ↓
    LOCAL CHRONO SERVICE API (v1)
    [127.0.0.1:8900]
        ├── HTTP: /api/v1/snapshot
        ├── WS:   /api/v1/stream
        └── HTTP: /api/v1/transaction/:sig
                ↓
    Browser UI / Downstream Clients
```

### Key Operating Invariants:
1. **Rust Core as Single Source of Truth**: Downstream clients (e.g. browser frontend) perform **zero direct network calls to Solana RPC or WebSockets**.
2. **Monotonic Event Sequencing**: Every event emitted by the service carries a service-local monotonic `sequence` number (1, 2, 3...) independent of Solana slot numbers.
3. **Sequence Replay & Gap Detection**: On reconnection, clients send `last_sequence`. If within the ring buffer capacity (default 2,048), missing events are replayed immediately; if a gap occurs, the server signals a `gap` message instructing the client to fetch a fresh snapshot.
4. **Honest Provenance**: Every metric and field is tagged with its empirical provenance (`DIRECT`, `DERIVED`, `ESTIMATED`, or `UNAVAILABLE`).

---

## 2. HTTP Endpoints

All HTTP endpoints support CORS and return JSON.

### 2.1 GET `/api/v1/health`
Health check endpoint for container probes and supervisor scripts.

**Response**:
```json
{
  "status": "healthy",
  "service": "chrono-server",
  "version": "0.1.0",
  "timestamp_ms": 1791000000000
}
```

---

### 2.2 GET `/api/v1/status`
Operational status of the Chrono service and its underlying source.

**Response**:
```json
{
  "status": "LIVE",
  "cluster": "testnet",
  "source": "rpc-ws",
  "environment": "live",
  "sequence": 1420,
  "client_count": 2,
  "uptime_ms": 86400000
}
```

---

### 2.3 GET `/api/v1/snapshot`
Returns the complete, authoritative consensus snapshot of the Solana cluster.

**Request**:
```bash
curl -s http://127.0.0.1:8900/api/v1/snapshot | jq .
```

**Response Schema (`ChronoSnapshotWire`)**:
```json
{
  "schema_version": 1,
  "sequence": 1420,
  "snapshot_timestamp_ms": 1791000000000,
  "cluster": "testnet",
  "source": "rpc-ws",
  "environment": "live",
  "status": "LIVE",
  "slot": {
    "current_slot": 341829100,
    "target_duration_ms": 250,
    "elapsed_ms": 118,
    "phase_ratio": 0.472,
    "provenance": "DIRECT"
  },
  "leader": {
    "current_leader": "7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELs8ZAKTUJ",
    "next_leader": "Gsf5P4QxW8n1YgY...9aB",
    "handoff_state": "ACTIVE",
    "provenance": "DIRECT"
  },
  "banks": {
    "candidate_banks": [
      {
        "bank_id": "bank-canonical",
        "raw_bank_id": null,
        "slot": 341829100,
        "parent_bank_id": null,
        "blockhash": "8xMvY3Z...8QfA",
        "bank_hash": null,
        "state": "CANONICAL",
        "tx_count": 1842,
        "observed_at_ms": 1791000000000,
        "provenance": "DIRECT",
        "abandonment_reason": null
      }
    ],
    "canonical_bank": {
      "bank_id": "bank-canonical",
      "slot": 341829100,
      "state": "CANONICAL"
    },
    "total_banks_tracked": 1
  },
  "parent": {
    "last_update_parent": null,
    "total_update_parents": 0
  },
  "finality": {
    "mode": "ALPENGLOW_VOTOR",
    "finality_latency_ms": 114,
    "last_finalized_slot": 341829099,
    "cert_type": "FINAL_CERT_FAST_PATH_80",
    "stake_percent": 88.4,
    "provenance": "DIRECT"
  },
  "capabilities": {
    "dimensions": {
      "slot": "SUPPORTED",
      "leader": "SUPPORTED",
      "bank_id": "UNSUPPORTED",
      "update_parent": "UNSUPPORTED",
      "bls_finality": "SUPPORTED"
    },
    "coverage_score": 60,
    "limitations": [
      "Public JSON-RPC does not expose validator-local bank_id integers",
      "Public JSON-RPC does not emit UpdateParent marker events before block seal"
    ]
  },
  "network": {
    "connected": true,
    "rpc_endpoint": "https://api.testnet.solana.com",
    "live_tps": 2840,
    "active_validators": 1940,
    "last_update_ms": 1791000000000
  },
  "telemetry": {
    "events_received_total": 5420,
    "events_normalized_total": 5420,
    "events_dropped_total": 0,
    "source_latency_us": 45000,
    "processing_latency_us": 2000,
    "websocket_clients": 1
  },
  "recent_events": []
}
```

---

### 2.4 GET `/api/v1/transaction/:signature`
Forensic transaction autopsy query. Evaluates whether a transaction succeeded, was dropped during a leader handoff, or was abandoned due to an `UpdateParent` event.

**Request**:
```bash
curl -s http://127.0.0.1:8900/api/v1/transaction/5xY...sig | jq .
```

**Response (`TransactionAutopsyResult`)**:
```json
{
  "signature": "5xY...sig",
  "slot": 341829050,
  "leader": "7Np41oeYqPefeNQEHSv1UDhYrehxin3NStELs8ZAKTUJ",
  "candidateBank": "bank-canonical",
  "parentRelation": "slot-341829049",
  "confirmationStatus": "CONFIRMED",
  "err": null,
  "evidence": [
    {
      "tier": "CORE_CANONICAL_CONFIRMED",
      "description": "Observed in canonical candidate bank on slot 341829050",
      "timestampMs": 1791000000000,
      "provenance": "DIRECT"
    }
  ]
}
```

---

## 3. WebSocket Streaming Protocol (`/api/v1/stream`)

The WebSocket interface delivers real-time normalized consensus events.

### 3.1 Handshake Lifecycle

1. **Client Connection**: Client connects to `ws://127.0.0.1:8900/api/v1/stream`.
2. **Client Hello**: Client MUST send a `hello` frame:
   ```json
   {
     "type": "hello",
     "schema_version": 1,
     "client_id": "client-browser-01",
     "last_sequence": 1400
   }
   ```
3. **Server Welcome**: Server responds with current state sequence:
   ```json
   {
     "type": "welcome",
     "client_id": "client-browser-01",
     "current_sequence": 1420,
     "snapshot_sequence": 1420
   }
   ```
4. **Replay vs Gap**:
   - If `last_sequence < current_sequence` and `last_sequence >= oldest_buffer_sequence`, server immediately transmits all missed events in sequence order.
   - If `last_sequence < oldest_buffer_sequence`, server transmits:
     ```json
     {
       "type": "gap",
       "expected_sequence": 1400,
       "oldest_available_sequence": 1410,
       "current_sequence": 1420
     }
     ```
     The client then triggers an HTTP `/api/v1/snapshot` refresh.

### 3.2 Real-time Event Frame
Server streams wrapped events:
```json
{
  "type": "event",
  "event": {
    "schema_version": 1,
    "sequence": 1421,
    "event_id": 1421,
    "cluster": "testnet",
    "source": "rpc-ws",
    "environment": "live",
    "observed_at_ms": 1791000000100,
    "received_at_ms": 1791000000102,
    "slot": 341829101,
    "bank_id": null,
    "blockhash": "9zKq3LtP...8QfA",
    "parent_slot": 341829100,
    "parent_blockhash": "8xMvY3Z...8QfA",
    "provenance": "DIRECT",
    "event_type": "SlotProgress",
    "payload": {
      "slot": 341829101,
      "elapsed_ms": 12,
      "target_duration_ms": 250
    }
  }
}
```

### 3.3 Heartbeats & Slow-Client Backpressure
- Server sends periodic ping frames (`{"type": "ping"}`) every 15 seconds. Client responds with `{"type": "pong"}`.
- If a client's outbound TCP channel buffer overflows (lagging consumer), the server terminates the connection to prevent memory exhaustion in the core daemon.

---

## 4. Error Codes & Handling

| HTTP Code | Error Condition | Action |
|-----------|-----------------|--------|
| `200 OK` | Successful query | Process payload |
| `400 Bad Request` | Malformed JSON or invalid query | Check request formatting |
| `404 Not Found` | Unknown endpoint | Verify URL matches `/api/v1/...` |
| `500 Internal Error` | Internal error in state machine | Inspect `chrono serve` stdout logs |
| `503 Service Unavailable` | Source connecting or cluster offline | Retry with exponential backoff |
