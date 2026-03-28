# platform-manager

A modular Rust platform manager that exposes application functionality through both a **CLI** and a **gRPC service**.

## Architecture

The project is structured as a Cargo workspace with the following crates:

| Crate | Description |
|---|---|
| `core` | Business logic, domain models and domain errors (no external dependencies) |
| `app` | Application service layer that orchestrates action execution |
| `transport` | Pure mapping functions between protobuf types and domain types |
| `grpc` | Tonic-based gRPC server implementation |
| `cli` | Clap-based CLI client that communicates with the server over gRPC |

The single source of truth for the service contract is the `proto/` directory.

## Project structure

```text
platform-manager/
├── Cargo.toml          # workspace root
├── proto/
│   ├── manager.proto   # InfoService and LifeCycle definitions
│   └── factory.proto   # FactoryService definition
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

Get platform information:

```bash
cargo run --bin cli -- info
```

With tabular output:

```bash
cargo run --bin cli -- --output table info
```

This command uses the dedicated gRPC service `InfoService` and method `Info(InfoRequest) -> InfoResponse`.

Request graceful termination of the server:

```bash
cargo run --bin cli -- terminate
```

Expected output shape:

```json
{
    "message": "termination requested"
}
```

This command uses the gRPC service `LifeCycle` and method `Terminate(TerminateRequest) -> TerminateResponse`.
After a successful call, the server receives a shutdown signal and stops gracefully.

Deploy another application using `deploy-agent`:

```bash
cargo run --bin cli -- deploy-agent agent.json
```

Where `agent.json` contains the deployment configuration:

```json
{
    "application": "orders-api",
    "url": "https://orders.example.com"
}
```

Expected output shape:

```json
{
    "application": "platform-manager",
    "endpoints": [
        {"name": "grpc_info_rpc", "value": "/manager.InfoService/Info (InfoRequest -> InfoResponse)"}
    ],
    "launched_applications": [
        {"application": "platform-manager", "url": "http://localhost:50051"},
        {"application": "orders-api", "url": "https://orders.example.com"}
    ],
    "task_id": "task-..."
}
```

### Run tests

```bash
cargo test --workspace
```
