# Payment Processing System

## Architecture

- **Event Sourcing**: All state changes captured as immutable events
- **Actor-per-Client**: Each client managed by isolated actor (ractor)

From an high level view, the engine is a stateless orchestrator that processes incoming transactions in the following way:

- Command Handlers: Responsible for validating and transforming commands into events

  1. **Load**: Fetches any required state/resources
  2. **Validate**: Ensures command adheres to business rules
  3. **Emit**: Produces one or more events based on the command
  4. **Effect**: Applies side effects (e.g., sending a message or whatever it may be)

- Event Handlers: Responsible for applying events to client state

The engine itself is responsible for sequencing commands/events, ensuring idempotency, and applying event to state.

Each client is represented by a globally unique actor, that manages its own in memory state. The actor receives commands from the engine,
applies them to its state, and returns the resulting events back to the engine for persistence.

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

I'm fully aware that this is **not production-ready.** There's a lot to be improved and perhaps re-architected. But as far as a toy project goes, it's enough.

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
- [ ] Graceful shutdown (Actors should store a snapshot before shutting down, ideally the behaviour should be like Pekko, where actors can be moved to a different node while the pod shutsdown so they survive deployment with state intact)
- [ ] Snapshot/restore
- [ ] Backpressure handling (Perhaps look into ractor Factory and design it in such a way that it does Hashing instead of individual actors, and before creating a client we just check for that globally unique worker that SHOULD have the client in its registry)

## Performance

real 38.71
user 18.68
sys 27.09
          1688584192  maximum resident set size
                   0  average shared memory size
                   0  average unshared data size
                   0  average unshared stack size
              175176  page reclaims
                   0  page faults
                   0  swaps
                   0  block input operations
                   0  block output operations
                   0  messages sent
                   0  messages received
                   0  signals received
                  95  voluntary context switches
            11241813  involuntary context switches
        263334116941  instructions retired
        148903237383  cycles elapsed
          1909114656  peak memory footprint

Benchmark: 3,750,587 transactions (140 MB CSV)

| Metric | Value |
|--------|-------|
| **Throughput** | 96,887 tx/sec |
| **Peak Memory** | 1.78 GiB |
| **Efficiency** | 96,887 tx/sec |
| **Processing Time** | 38.71 seconds |
| **Context Switches** | 11.2M involuntary, 95 voluntary |

Memory scales O(1) per client - processing 45x more data than initial tests with only 74% memory usage. For the reader,
this is using InMemoryJournal, which makes the memory usage grow quite drastically.
