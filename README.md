# Payment Processing System

## Architecture

- **Event Sourcing**: All state changes captured as immutable events
- **Actor-per-Client**: Each client managed by isolated actor (ractor)

## Components

| Component | Responsibility |
|-----------|---------------|
| **Engine** | Command/Event orchestration (`load → validate → emit → effect`) |
| **Journal** | Event persistence, idempotency, sequence numbers |
| **ClientActor** | State management, ordering guarantees, serialization |
| **ClientRegistry** | Actor lifecycle, distributed lookup (ractor global registry) |

## Usage

```bash
# Process transactions
cargo run --release -- transactions.csv > accounts.csv

# Generate test data
cargo run --release -- generate -n 100 -c 5 test.csv

# Run tests
cargo test
```

## Design Decisions

### Frozen Account Behavior

| Operation | Allowed? | Rationale |
|-----------|----------|-----------|
| **Deposit** | ✅ Yes | Incoming funds (refunds/credits) permitted |
| **Withdrawal** | ❌ No | Outgoing funds blocked |
| **Dispute/Resolve/Chargeback** | ✅ Yes | Consumer protection - dispute resolution continues |

## Disclaimer

I'm fully away that this **not production-ready.** There's a lot be improved and perhaps re-architected. But as far as
a toy project goes its enough.

**Infrastructure**:
- [ ] Persistent storage (PostgreSQL/Cassandra for journal)
- [ ] TLS for inter-node communication (ractor_cluster)
- [ ] Service discovery (Kubernetes)
- [ ] Proper configuration management (env vars, secrets)

**Resilience**:
- [ ] Circuit breakers for external dependencies
- [ ] Retry policies with exponential backoff
- [ ] Dead letter queue for failed events
- [ ] Event replay mechanisms (temporal queries)

**Security**:
- [ ] Authentication/Authorization (Cookie ractor_cluster)
- [ ] Audit logging
- [ ] Encryption at rest

**Observability**:
- [ ] Structured logging (tracing integration)
- [ ] Metrics (Prometheus)
- [ ] Distributed tracing (OpenTelemetry)
- [ ] Health checks and readiness probes
- [ ] Alerting

**Operations**:
- [ ] Graceful shutdown (Actors should store a snapshot before shutting down, ideally the behaviour should be like Pekko,
where actors can be moved to a different node while the pod shutsdown so they survive deployment with state intact)
- [ ] Snapshot/restore
- [ ] Backpressure handling (Perhaps look into ractor Factory and design it in such a way that it does Hashing instead of individual actors, and before creating a client we just check for that globally unique worker that SHOULD have the client in its registry)