# platform-manager

A modular Rust platform manager that exposes application functionality through both a **CLI** and a **gRPC service**.

## Architecture

The project is structured as a Cargo workspace with the following crates:

| Crate | Description |
|---|---|
| `core` | Business logic, action trait, registry and domain errors (no external dependencies) |
| `app` | Application service layer that orchestrates action execution |
| `transport` | Pure mapping functions between protobuf types and domain types |
| `grpc` | Tonic-based gRPC server implementation |
| `cli` | Clap-based CLI client that communicates with the server over gRPC |

The single source of truth for the service contract is `proto/action.proto`.

## Project structure

```text
platform-manager/
├── Cargo.toml          # workspace root
├── proto/
│   └── action.proto    # gRPC service definition
├── crates/
│   ├── core/           # business logic and domain types
│   ├── app/            # application orchestration layer
│   ├── transport/      # protobuf ↔ domain mapping
│   ├── grpc/           # gRPC server
│   └── cli/            # CLI client
└── bin/
    ├── server.rs       # server entry point
    └── cli.rs          # CLI entry point
```

## Technologies

- [`tokio`](https://tokio.rs/) — async runtime
- [`tonic`](https://github.com/hyperium/tonic) — gRPC framework
- [`prost`](https://github.com/tokio-rs/prost) — protobuf code generation
- [`clap`](https://docs.rs/clap) — CLI argument parsing
- [`serde`](https://serde.rs/) / `serde_json` — JSON serialization
- [`tracing`](https://docs.rs/tracing) — structured logging
- [`thiserror`](https://docs.rs/thiserror) / [`anyhow`](https://docs.rs/anyhow) — error handling

## Getting started

### Build

```bash
cargo build --workspace
```

### Run the server

```bash
cargo run --bin server
```

### Run the CLI

```bash
cargo run --bin cli -- execute echo '{"message":"hello"}'
```

Expected output:

```json
{"message":"hello"}
```

### Run tests

```bash
cargo test --workspace
```
