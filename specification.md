# Specifica Architetturale — Progetto Rust CLI + gRPC

## 1. Obiettivo

Realizzare un progetto Rust basato su architettura modulare che esponga funzionalità applicative tramite:

- **CLI** per utilizzo interattivo e scripting
- **servizio gRPC** per integrazione machine-to-machine

L’architettura deve essere progettata per:

- massima separazione delle responsabilità
- forte tipizzazione tramite **Protocol Buffers**
- singolo punto di verità per il contratto di servizio
- elevata testabilità
- facilità di estensione con nuove azioni

Il documento è scritto in modo da poter essere usato come **input per la generazione automatica del progetto da parte di un modello AI**.

---

## 2. Requisiti architetturali

### Requisiti funzionali

Il sistema deve consentire:

1. esecuzione di azioni tramite comando CLI
2. esecuzione delle stesse azioni tramite endpoint gRPC
3. condivisione della stessa logica applicativa tra CLI e server
4. aggiunta di nuove azioni senza modificare l’infrastruttura esistente

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
│   └── action.proto
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
- definizione delle azioni
- errori di dominio
- trait comuni

Non deve dipendere da:

- tonic
- clap
- tokio
- protoc generated types

### API richiesta

```rust
pub trait Action: Send + Sync {
    fn name(&self) -> &'static str;

    fn execute(
        &self,
        input: Vec<u8>,
    ) -> Result<Vec<u8>, AppError>;
}
```

Deve includere anche:

```rust
pub struct ActionRegistry {
    // mappa nome azione -> implementazione
}
```

Funzioni richieste:

- `register()`
- `get()`
- `execute()`

---

### 5.2 crate `app`

Layer di orchestrazione.

Responsabile di:

- invocare il registry
- gestire workflow applicativi
- validazione input applicativa
- mapping errori di alto livello

API richiesta:

```rust
pub struct AppService {
    registry: ActionRegistry,
}

impl AppService {
    pub fn execute(
        &self,
        action: &str,
        payload: Vec<u8>,
    ) -> Result<Vec<u8>, AppError>
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
pub fn from_proto(req: ActionRequest) -> (String, Vec<u8>);
pub fn to_proto(res: Result<Vec<u8>, AppError>) -> ActionResponse;
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

Deve esporre servizio:

```rust
ActionServiceServer
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
my_app execute compress '{"path":"file.txt"}'
```

---

## 6. Specifica protobuf

Generare file:

`proto/action.proto`

Contenuto richiesto:

```proto
syntax = "proto3";

package action;

service ActionService {
  rpc Execute(ActionRequest) returns (ActionResponse);
}

message ActionRequest {
  string action = 1;
  bytes payload = 2;
}

message ActionResponse {
  bytes payload = 1;
  string error = 2;
}
```

Questo file è il **single source of truth**.

---

## 7. build.rs

Il crate gRPC deve includere `build.rs`.

```rust
fn main() {
    tonic_build::compile_protos("../../proto/action.proto")
        .unwrap();
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
ActionRegistry
 ↓
Concrete Action
```

---

## 9. Regole di implementazione

### Regola 1

La logica business non deve mai dipendere dal transport layer.

### Regola 2

I tipi protobuf non devono essere usati nel crate `core`.

### Regola 3

Ogni nuova azione deve implementare `Action`.

### Regola 4

La CLI deve sempre usare gRPC, senza accesso diretto al core.

---

## 10. Azione di esempio obbligatoria

Generare almeno una action di esempio:

`echo`

Comportamento:

input payload JSON:

```json
{
  "message": "hello"
}
```

output:

```json
{
  "message": "hello"
}
```

---

## 11. Test richiesti

Generare test per:

- `core` unit tests
- `app` service tests
- gRPC integration tests
- CLI smoke tests

---

## 12. Output atteso dalla generazione AI

Il modello AI deve generare:

1. workspace completo
2. `Cargo.toml` root
3. `Cargo.toml` per ogni crate
4. file `action.proto`
5. implementazione server
6. implementazione client CLI
7. action di esempio
8. test base
9. README di esecuzione

---

## 13. Prompt operativo per generazione

Usare questa istruzione come prompt di generazione:

> Genera un workspace Rust multi-crate conforme a questa specifica, completo di codice compilabile, test base e action di esempio `echo`, usando tonic, tokio, clap e protobuf come source of truth.



---

## 14. Convenzioni di naming e struttura del codice

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
└── echo.rs
```

### Naming tipi

Usare i seguenti suffissi:

- `*Request`
- `*Response`
- `*Service`
- `*Error`
- `*Action`

Esempi:

- `EchoAction`
- `AppService`
- `GrpcActionService`
- `AppError`

---

## 15. Standard di error handling

### Dominio

Nel crate `core` usare esclusivamente `thiserror`.

Esempio:

```rust
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("action not found: {0}")]
    ActionNotFound(String),

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

## 16. Logging e observability

Integrare `tracing` come standard.

Requisiti obbligatori:

- log startup server
- log richiesta ricevuta
- log azione eseguita
- log errore
- log shutdown

Esempio:

```rust
tracing::info!(action = action_name, "executing action");
```

Il `main` del server deve inizializzare:

```rust
tracing_subscriber::fmt::init();
```

---

## 17. Standard di testing

### Unit test

Ogni action deve avere unit test dedicati.

Pattern richiesto:

```rust
#[cfg(test)]
mod tests
```

Coverage minimo richiesto:

- success path
- invalid payload
- action non trovata

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
cargo run --bin cli -- execute echo '{"message":"hello"}'
```

---

## 18. Criteri di acceptance

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
cargo run --bin cli -- execute echo '{"message":"hello"}'
```

### Risultato atteso

Output JSON:

```json
{
  "message": "hello"
}
```

---

## 19. Estendibilità futura

L'architettura deve essere predisposta per futura estensione con:

- autenticazione gRPC
- TLS
- plugin action dinamiche
- streaming gRPC
- health check endpoint
- metrics Prometheus

Non implementare tali feature ora, ma predisporre la struttura per supportarle.

---

## 20. Prompt finale ottimizzato per AI code generation

Usare il seguente prompt finale.

> Genera un workspace Rust multi-crate production-ready conforme a questa specifica. Il codice deve essere compilabile, idiomatico, testabile, con separazione rigorosa tra core, application layer, transport, gRPC server e CLI client. Usa tonic, tokio, prost, clap, tracing, thiserror e anyhow. Includi unit test, integration test e una action di esempio echo completamente funzionante.



---

## 21. Checklist di generazione sequenziale (generator-friendly)

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

Generare `proto/action.proto` esattamente secondo la specifica.

Acceptance:

- sintassi `proto3`
- package `action`
- service `ActionService`
- RPC `Execute`

---

### Step 3 — Code generation setup

Generare `build.rs` nel crate gRPC.

Acceptance:

- `tonic_build` configurato
- `cargo build` genera i tipi protobuf

---

### Step 4 — Implementazione crate core

Generare:

- trait `Action`
- `ActionRegistry`
- `AppError`
- action `EchoAction`

Acceptance:

- nessuna dipendenza da tonic/clap
- unit test presenti

---

### Step 5 — Implementazione application layer

Generare `AppService`.

Acceptance:

- orchestration corretta
- delega al registry
- gestione errori tipizzati

---

### Step 6 — Implementazione transport layer

Generare mapping funzioni:

- `from_proto()`
- `to_proto()`

Acceptance:

- mapping puro
- nessuna logica business

---

### Step 7 — Implementazione server gRPC

Generare:

- implementazione trait tonic
- bootstrap server
- logging

Acceptance:

- server avviabile
- request gestite correttamente

---

### Step 8 — Implementazione CLI client

Generare:

- parser clap
- subcommand `execute`
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

## 22. Strategia di generazione consigliata per AI

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

