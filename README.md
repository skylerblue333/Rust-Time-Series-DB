# Sky Time Series — Rust Engineering Beta

Sky Time Series is a focused Rust/Actix Web service for accepting numeric time-series points into an in-memory store and querying them by key and timestamp range.

## Status

**Engineering beta.** The current implementation has a real Rust store, validation, deterministic timestamp ordering, range queries, health/readiness endpoints, unit tests, CI, dependency auditing, and a non-root container. The in-memory store is currently **unbounded**: it has no capacity, eviction, or retention policy, so sustained writes can grow process memory. It is **not** a durable database and does not claim replication, WAL persistence, compaction, retention policies, clustering, HA, multi-tenancy, or production deployment.

## API

- `POST /api/v1/points` — insert `{ "key": "cpu", "value": 72.4, "timestamp": 1724450000 }`
- `GET /api/v1/series/{key}?start=<u64>&end=<u64>` — inclusive range query
- `GET /healthz` — process health
- `GET /readyz` — store readiness and current point count

## Run locally

```bash
cargo run
```

Set `BIND_ADDR` to override the default `0.0.0.0:8080` bind address.

## Verify

```bash
cargo fmt --all -- --check
cargo check --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo audit
docker build -t sky-timeseries .
docker run --rm --entrypoint=id sky-timeseries -u
```

The container is expected to run as UID `10001`, not root.

## Architecture

`src/lib.rs` contains the reusable in-memory store. Each series is a vector of validated records held behind `Arc<Mutex<...>>`; records are sorted by timestamp after insertion. `src/main.rs` exposes the store through a small Actix Web API. This design is intentionally simple and suitable for a reusable engineering component, not a durable TSDB replacement.

## SKYCOIN4444 integration

Use this service behind a stable HTTP interface for short-lived metrics, simulations, demos, or development telemetry. For durable ecosystem analytics, route writes to a persistent datastore instead of treating this beta service as authoritative storage.

## Security and operational boundaries

The service validates key length, finite numeric values, and query ranges, but it does not currently provide authentication, authorization, TLS termination, rate limiting, persistence, encryption at rest, tenant isolation, or memory-retention controls. Deploy only behind appropriate infrastructure if used outside local development.

## License

See `LICENSE`.
