# Specifica Architetturale — Progetto Rust CLI + gRPC

## 1. Obiettivo

Realizzare un progetto Rust basato su architettura modulare che esponga funzionalità applicative tramite:

- **CLI** per utilizzo interattivo e scripting
- **servizio gRPC** per integrazione machine-to-machine

L'architettura deve essere progettata per:

- massima separazione delle responsabilità
- forte tipizzazione tramite **Protocol Buffers**
- singolo punto di verità per il contratto di servizio
- elevata testabilità
- facilità di estensione con nuove funzionalità

Il documento è scritto in modo da poter essere usato come **input per la generazione automatica del progetto da parte di un modello AI**.

---

## 2. Requisiti architetturali

### Requisiti funzionali

Il sistema deve consentire:

1. esecuzione di operazioni tramite comando CLI
2. esecuzione delle stesse operazioni tramite endpoint gRPC dedicati
3. condivisione della stessa logica applicativa tra CLI e server

### Requisiti non funzionali

- codice idiomatico Rust
- workspace multi-crate
- supporto async con Tokio
- logging strutturato
- error handling consistente
- separazione netta tra transport e dominio

---

## 3. Tecnologie obbligatorie

Il progetto deve usare obbligatoriamente:

- `tokio` come async runtime
- `tonic` per gRPC
- `prost` per code generation protobuf
- `clap` per CLI
- `serde` e `serde_json` per serializzazione payload
- `tracing` per logging
- `thiserror` per errori di dominio
- `anyhow` per errori applicativi

---

## 4. Struttura del progetto

Generare il seguente workspace Cargo.

```text
my_app/
├── Cargo.toml
├── proto/
│   ├── manager.proto
│   └── factory.proto
├── crates/
│   ├── core/
│   ├── app/
│   ├── transport/
│   ├── grpc/
│   └── cli/
└── bin/
    ├── server.rs
    └── cli.rs
```

---

## 5. Responsabilità dei moduli

### 5.1 crate `core`

Contiene esclusivamente:

- logica di business
- definizione delle operazioni
- errori di dominio
- modelli comuni

Non deve dipendere da:

- tonic
- clap
- tokio
- protoc generated types

---

### 5.2 crate `app`

Layer di orchestrazione.

Responsabile di:

- invocare le operazioni di business
- gestire workflow applicativi
- validazione input applicativa
- mapping errori di alto livello

API richiesta:

```rust
pub struct AppService {
    info_action: InfoAction,
    deploy_agent_action: DeployAgentAction,
}

impl AppService {
    pub fn get_info(&self) -> Result<Vec<u8>, AppError>;
    pub fn deploy_agent(&self, payload: Vec<u8>) -> Result<Vec<u8>, AppError>;
}
```

---

### 5.3 crate `transport`

Responsabile del mapping tra:

- tipi protobuf
- tipi di dominio

Deve contenere funzioni pure.

Esempio:

```rust
pub fn to_info_proto(res: Result<Vec<u8>, AppError>) -> InfoResponse;
```

---

### 5.4 crate `grpc`

Contiene il server gRPC.

Responsabilità:

- bootstrap server
- implementazione trait tonic generated
- gestione request/response
- traduzione errori gRPC

Il server deve usare:

```rust
#[tonic::async_trait]
```

Deve esporre i servizi:

```rust
InfoServiceServer
FactoryServiceServer
LifeCycleServer
```

---

### 5.5 crate `cli`

Client CLI basato su `clap`.

Responsabilità:

- parsing argomenti
- connessione al server gRPC
- invio request
- rendering output

Deve usare subcommand pattern.

Esempio:

```rust
my_app info
my_app deploy-agent agent.json
my_app terminate
```

---

## 6. Specifica protobuf

Generare i file:

`proto/manager.proto` — InfoService e LifeCycle

```proto
syntax = "proto3";
package manager;

service InfoService {
  rpc Info(InfoRequest) returns (InfoResponse);
}

service LifeCycle {
  rpc Terminate(TerminateRequest) returns (TerminateResponse);
}
```

`proto/factory.proto` — FactoryService

```proto
syntax = "proto3";
package factory;

service FactoryService {
  rpc DeployAgent(DeployAgentRequest) returns (DeployAgentResponse);
}
```

Questi file sono il **single source of truth**.

---

## 7. build.rs

Il crate transport deve includere `build.rs`.

```rust
fn main() {
    tonic_build::configure().compile_protos(
        &["../../proto/manager.proto", "../../proto/factory.proto"],
        &["../../proto"],
    ).unwrap();
}
```

---

## 8. Flusso di esecuzione

```text
CLI
 ↓
gRPC Client
 ↓
Tonic Server
 ↓
Transport Mapper
 ↓
AppService
 ↓
Concrete Operation (InfoAction / DeployAgentAction)
```

---

## 9. Regole di implementazione

### Regola 1

La logica business non deve mai dipendere dal transport layer.

### Regola 2

I tipi protobuf non devono essere usati nel crate `core`.

### Regola 3

La CLI deve sempre usare gRPC, senza accesso diretto al core.

### Regola 4

Ogni operazione esposta deve avere un endpoint gRPC dedicato con tipi fortemente tipizzati.

---

## 10. Test richiesti

Generare test per:

- `core` unit tests
- `app` service tests
- gRPC integration tests
- CLI smoke tests

---

## 11. Output atteso dalla generazione AI

Il modello AI deve generare:

1. workspace completo
2. `Cargo.toml` root
3. `Cargo.toml` per ogni crate
4. file `manager.proto` e `factory.proto`
5. implementazione server
6. implementazione client CLI
7. test base
8. README di esecuzione

---

## 12. Prompt operativo per generazione

Usare questa istruzione come prompt di generazione:

> Genera un workspace Rust multi-crate conforme a questa specifica, completo di codice compilabile e test base, usando tonic, tokio, clap e protobuf come source of truth. Ogni operazione deve avere un endpoint gRPC dedicato.



---

## 13. Convenzioni di naming e struttura del codice

Per massimizzare la qualità della generazione automatica, applicare le seguenti convenzioni.

### Naming crate

Usare nomi espliciti e consistenti:

- `my_app_core`
- `my_app_app`
- `my_app_transport`
- `my_app_grpc`
- `my_app_cli`

### Naming moduli

Ogni crate deve seguire questa struttura minima:

```text
src/
├── lib.rs
├── errors.rs
├── models.rs
└── service.rs
```

Nel crate `core` aggiungere:

```text
src/actions/
├── mod.rs
├── info.rs
├── deploy_agent.rs
└── launched_apps.rs
```

### Naming tipi

Usare i seguenti suffissi:

- `*Request`
- `*Response`
- `*Service`
- `*Error`
- `*Action`

Esempi:

- `InfoAction`
- `DeployAgentAction`
- `AppService`
- `GrpcInfoService`
- `AppError`

---

## 14. Standard di error handling

### Dominio

Nel crate `core` usare esclusivamente `thiserror`.

Esempio:

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("invalid payload")]
    InvalidPayload,

    #[error("execution failed: {0}")]
    Execution(String),
}
```

### Application / bootstrap

Usare `anyhow::Result` solo nei binari e nel bootstrap.

Regola obbligatoria:

- `core`: errori tipizzati
- `app`: errori tipizzati
- `bin`: `anyhow`

---

## 15. Logging e observability

Integrare `tracing` come standard.

Requisiti obbligatori:

- log startup server
- log richiesta ricevuta
- log operazione eseguita
- log errore
- log shutdown

Esempio:

```rust
tracing::info!("executing info action");
```

Il `main` del server deve inizializzare:

```rust
tracing_subscriber::fmt::init();
```

---

## 16. Standard di testing

### Unit test

Ogni operazione deve avere unit test dedicati.

Pattern richiesto:

```rust
#[cfg(test)]
mod tests
```

Coverage minimo richiesto:

- success path
- invalid payload

### Integration test gRPC

Generare cartella:

```text
tests/
└── grpc_execute.rs
```

Test obbligatori:

- avvio server
- invocazione client
- verifica response

### CLI smoke test

Test di esecuzione comando:

```bash
cargo run --bin cli -- info
```

---

## 17. Criteri di acceptance

Il progetto generato è accettato solo se soddisfa tutti i seguenti criteri.

### Build

```bash
cargo build --workspace
```

Deve completarsi senza warning critici.

### Test

```bash
cargo test --workspace
```

Tutti i test devono passare.

### Runtime

Server avviabile con:

```bash
cargo run --bin server
```

CLI invocabile con:

```bash
cargo run --bin cli -- info
```

### Risultato atteso

Output JSON con informazioni sulla piattaforma e le applicazioni avviate.

---

## 18. Estendibilità futura

L'architettura deve essere predisposta per futura estensione con:

- autenticazione gRPC
- TLS
- streaming gRPC
- health check endpoint
- metrics Prometheus

Non implementare tali feature ora, ma predisporre la struttura per supportarle.

---

## 19. Prompt finale ottimizzato per AI code generation

Usare il seguente prompt finale.

> Genera un workspace Rust multi-crate production-ready conforme a questa specifica. Il codice deve essere compilabile, idiomatico, testabile, con separazione rigorosa tra core, application layer, transport, gRPC server e CLI client. Usa tonic, tokio, prost, clap, tracing, thiserror e anyhow. Includi unit test e integration test. Ogni operazione esposta deve avere un endpoint gRPC dedicato con tipi fortemente tipizzati.



---

## 20. Checklist di generazione sequenziale (generator-friendly)

Questa sezione è ottimizzata per modelli AI che lavorano meglio con task step-by-step.

Il generatore deve eseguire i task **in ordine rigoroso**.

### Step 1 — Creazione workspace

Generare la struttura root:

```text
my_app/
├── Cargo.toml
├── proto/
├── crates/
└── bin/
```

Acceptance:

- workspace compilabile
- membri correttamente dichiarati

---

### Step 2 — Definizione contratto protobuf

Generare `proto/manager.proto` e `proto/factory.proto` esattamente secondo la specifica.

Acceptance:

- sintassi `proto3`
- servizi dedicati per ogni operazione
- tipi fortemente tipizzati

---

### Step 3 — Code generation setup

Generare `build.rs` nel crate transport.

Acceptance:

- `tonic_build` configurato
- `cargo build` genera i tipi protobuf

---

### Step 4 — Implementazione crate core

Generare:

- `InfoAction`
- `DeployAgentAction`
- `LaunchedApps`
- `AppError`

Acceptance:

- nessuna dipendenza da tonic/clap
- unit test presenti

---

### Step 5 — Implementazione application layer

Generare `AppService`.

Acceptance:

- metodi specifici per ogni operazione
- gestione errori tipizzati

---

### Step 6 — Implementazione transport layer

Generare mapping funzioni:

- `to_info_proto()`

Acceptance:

- mapping puro
- nessuna logica business

---

### Step 7 — Implementazione server gRPC

Generare:

- implementazione trait tonic per ogni servizio
- bootstrap server
- logging

Acceptance:

- server avviabile
- request gestite correttamente

---

### Step 8 — Implementazione CLI client

Generare:

- parser clap
- subcommand per ogni operazione
- client gRPC

Acceptance:

- connessione al server
- output JSON leggibile

---

### Step 9 — Test end-to-end

Generare test completi.

Acceptance:

- unit test
- integration test
- CLI smoke test
- `cargo test --workspace` verde

---

## 21. Strategia di generazione consigliata per AI

Il modello deve generare il progetto seguendo la seguente strategia:

1. creare skeleton e file vuoti
2. completare i `Cargo.toml`
3. generare protobuf
4. implementare core
5. implementare server
6. implementare client
7. aggiungere test
8. validare compilazione finale

Regola importante:

> non generare tutto in un singolo blocco monolitico; procedere per step incrementali e coerenti.
