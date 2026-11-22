# FinSight AI

**AI-Assisted Financial Crime Investigation Platform**

FinSight AI is an educational, portfolio-focused full-stack software
engineering project for financial transaction monitoring, suspicious
activity investigation, and financial crime case management.

The platform combines a Rust backend, PostgreSQL persistence, a
React/TypeScript investigation interface, explainable risk analysis, and
evidence-grounded AI assistance. It is being developed incrementally as
part of a 100-day engineering roadmap.

> **Project status:** Core backend, frontend, AI investigation,
> Docker-based local infrastructure, and an initial blockchain
> evidence-integrity foundation have been implemented. Cloud deployment
> verification and the complete evidence-anchoring workflow remain in
> progress.

## Contents

-   [Implemented Features](#implemented-features)
-   [Technology Stack](#technology-stack)
-   [Architecture](#architecture)
-   [Security Model](#security-model)
-   [Blockchain Evidence
    Integrity](#blockchain-evidence-integrity--day-99-foundation)
-   [Project Structure](#project-structure)
-   [Local Development](#local-development)
-   [Testing](#testing)
-   [Roadmap and Current Status](#roadmap-and-current-status)
-   [Disclaimer](#disclaimer)

## Implemented Features

### Backend and API

-   REST API built with Rust, Axum, and Tokio.
-   PostgreSQL persistence through SQLx.
-   User registration and login.
-   Password hashing with Argon2.
-   JWT authentication and role-based authorization.
-   Case management and investigator assignment.
-   Financial transaction models, validation, and normalization.
-   Transaction ingestion and CSV bulk import.
-   Transaction-to-case linking.
-   Evidence metadata management and evidence package generation.
-   Health endpoints and HTTP request tracing.

### Financial Investigation

-   Configurable rule-based risk scoring.
-   Explainable risk indicators and risk-level classification.
-   Transaction graph construction and analysis.
-   Incoming/outgoing relationship analysis and multi-hop transaction
    paths.
-   Evidence-aware investigation context.

### AI-Assisted Investigation

-   LLM provider integration through Requesty.
-   Evidence-grounded investigation prompts.
-   Structured AI report output and validation.
-   Validation of references against supplied case evidence and
    transactions.
-   Markdown investigation report generation.
-   PostgreSQL persistence for investigation reports.

AI output is treated as investigative assistance. It must not invent
missing evidence or claim that a crime has occurred.

### Frontend

-   React 19 and TypeScript application built with Vite.
-   Authentication and bearer-token handling.
-   Case dashboard and investigation interface.
-   Transaction explorer.
-   Transaction graph visualization.
-   Shared frontend API services and types.

### Infrastructure

-   Multi-stage Docker builds for the Rust backend.
-   Dockerized frontend and Nginx reverse proxy.
-   Docker Compose local stack with PostgreSQL, backend, and frontend.
-   Database and application health checks.
-   Environment-based application configuration.
-   AWS EC2 and managed PostgreSQL deployment planning and
    infrastructure work.

## Technology Stack

  Area                    Technologies
  ----------------------- -----------------------------------------
  Backend                 Rust 2024, Axum 0.8, Tokio
  Database                PostgreSQL, SQLx
  Authentication          JWT, Argon2
  API and serialization   REST, Serde, JSON
  Frontend                React 19, TypeScript, Vite
  AI integration          Requesty LLM API
  Observability           tracing, tracing-subscriber, tower-http
  Containers              Docker, Docker Compose, Nginx
  Cloud target            AWS EC2, managed PostgreSQL
  Smart contracts         Solidity, Foundry
  Evidence hashing        SHA-256 (`sha2`)

## Architecture

At a high level, the application is organized as:

``` text
                    Investigator
                         |
                         v
              React + TypeScript UI
                         |
                    Nginx / API
                         |
                         v
                  Rust + Axum API
                   /     |      \
                  /      |       \
                 v       v        v
            PostgreSQL  Risk     AI provider
              storage  engine   (Requesty)
                 |
                 v
          Evidence metadata
                 |
          [Planned integration]
                 v
        Secure off-chain evidence
                 |
              SHA-256
                 |
                 v
       Blockchain evidence anchor
```

The local containerized stack uses Docker Compose to run PostgreSQL, the
backend API, and the frontend. Nginx handles frontend delivery and API
proxying.

## Security Model

Security controls implemented in the application include:

-   JWT-based authentication.
-   Argon2 password hashing.
-   Role-based authorization policies.
-   Request and domain-model validation.
-   Database constraints and foreign keys.
-   Restricted transaction ingestion.
-   Restricted evidence creation and sensitive investigation operations.

**Authorization rule:** Transaction ingestion is restricted to
`SENIOR_INVESTIGATOR` and `ADMIN`. Evidence creation and other protected
investigation operations also have role-based policy checks. Public
registration receives the ordinary investigator role.

Security practices for development and deployment:

-   Do not commit `.env` files, API keys, database passwords, wallet
    keystores, or private keys.
-   Keep database services and backend application ports private in
    production.
-   Use least-privilege security groups and IAM permissions.
-   Do not place sensitive financial documents or personal information
    on a public blockchain.
-   Treat AI-generated output as untrusted until validated against
    supplied evidence.

## Blockchain Evidence Integrity --- Day 99 Foundation

An initial Solidity evidence-anchoring contract and Rust SHA-256 utility
have been added to the repository.

### Implemented Foundation

-   Rust SHA-256 utility for hashing supplied bytes.
-   Unit tests for known SHA-256 values, empty input, output length, and
    differing inputs.
-   Solidity `EvidenceAnchor` contract.
-   Authorized recorder access control.
-   Owner-controlled recorder updates.
-   Duplicate hash rejection.
-   Rejection of empty hashes and invalid case IDs.
-   Read-only evidence-hash verification function.
-   Foundry tests for the contract's authorization and integrity rules.

### Verification Recorded During Development

-   Rust backend test suite: **144 passed, 0 failed**.
-   EvidenceAnchor Foundry tests: **8 passed, 0 failed**.
-   Full Foundry suite at that checkpoint: **10 passed, 0 failed**.

These are test results from the development checkpoint. Rerun the
commands against the current checkout to verify the latest state.

### Remaining Blockchain Work

The blockchain feature is **not yet end-to-end complete**. Remaining
tasks include:

1.  Implement protected evidence file upload.
2.  Store original evidence securely off-chain.
3.  Calculate the hash in the backend from the actual uploaded bytes; do
    not trust a client-supplied hash.
4.  Persist the calculated hash and storage reference.
5.  Deploy the contract to Ethereum Sepolia.
6.  Integrate Rust with the deployed contract and submit anchor
    transactions.
7.  Persist chain ID, contract address, transaction hash, and anchor
    status.
8.  Verify evidence by re-hashing the retrieved original file and
    comparing it with the database and on-chain record.

Only cryptographic commitments and the minimum necessary reference data
should be anchored on-chain. Evidence documents and sensitive financial
information must remain off-chain.

## Project Structure

``` text
FinSightAI/
├── backend/
│   ├── migrations/
│   ├── src/
│   │   ├── ai/
│   │   ├── auth/
│   │   ├── models/
│   │   ├── repositories/
│   │   ├── risk/
│   │   ├── services/
│   │   └── routes.rs
│   ├── Cargo.toml
│   └── Cargo.lock
├── frontend/
│   ├── src/
│   └── ...
├── blockchain/
│   ├── src/
│   │   └── EvidenceAnchor.sol
│   ├── test/
│   │   └── EvidenceAnchor.t.sol
│   ├── script/
│   ├── foundry.toml
│   └── foundry.lock
├── compose.yaml
└── README.md
```

## Local Development

### Prerequisites

-   Rust toolchain and Cargo.
-   PostgreSQL, or Docker Engine with the Docker Compose plugin.
-   Node.js and npm.
-   Foundry (for Solidity contract development and tests).

### Run with Docker Compose

Configure the root `.env` file with the PostgreSQL variables expected by
`compose.yaml`, and configure `backend/.env` with the backend
application settings. Use local secret values; never commit these files.

Start the stack:

``` bash
docker compose up --build -d
docker compose ps
```

The Compose configuration maps the frontend to `127.0.0.1:8080`. Open:

``` text
http://localhost:8080
```

Stop the stack:

``` bash
docker compose down
```

To also remove the local PostgreSQL volume, use `docker compose down -v`
only when you intentionally want to delete the local database data.

### Run the Backend Directly

Configure the backend environment, then:

``` bash
cd backend
cargo run
```

Run backend tests:

``` bash
cargo test
```

### Run the Frontend Directly

``` bash
cd frontend
npm install
npm run dev
```

Use the frontend scripts defined in `frontend/package.json` for builds
and checks.

### Build and Test the Smart Contract

``` bash
cd blockchain
forge build
forge test -vv
```

The contract has local Foundry tests. A successful local test does not
mean the contract has been deployed to Sepolia.

## Testing

Useful verification commands:

  Component              Command
  ---------------------- -----------------------------------
  Rust backend           `cd backend && cargo test`
  Solidity build         `cd blockchain && forge build`
  Solidity tests         `cd blockchain && forge test -vv`
  Compose services       `docker compose ps`
  Compose logs           `docker compose logs --tail=100`
  Git whitespace check   `git diff --check`

Test counts above describe a previous development checkpoint, not a
guarantee that every environment or deployment is currently healthy.

## Roadmap and Current Status

The project follows a 100-day roadmap. Recent milestones include:

-   **Days 85--91:** React and TypeScript frontend.
-   **Days 92--95:** Docker, Docker Compose, production configuration,
    health checks, and observability.
-   **Days 96--98:** Cloud architecture, AWS infrastructure, and
    deployment work.
-   **Days 99--100:** Blockchain evidence integrity and final production
    release.

Current outstanding work includes completing and verifying the live
cloud deployment, implementing the full evidence upload/storage/hash
workflow, deploying the evidence contract to Sepolia, integrating
blockchain transactions with the Rust backend, and completing the final
security review and portfolio documentation.

## Disclaimer

FinSight AI is an educational and portfolio software engineering project
under active development. It is not a certified AML compliance solution
and must not be used for real financial crime investigations without
appropriate security, legal, compliance, and operational validation.

## Author

**Metempsychosis16**

GitHub: https://github.com/Metempsychosis16

