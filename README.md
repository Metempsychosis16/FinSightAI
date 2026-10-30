# FinSight AI

**AI-Assisted Financial Crime Investigation Platform**

FinSight AI is an ongoing software engineering project focused on financial transaction monitoring, suspicious activity investigation, and financial crime case management.

The goal is to build a secure backend and investigation platform using Rust, PostgreSQL, and modern web technologies.

> **Status:** Under active development.

## Project Goals

- Build a structured financial investigation workflow.
- Store and manage financial transactions.
- Support transaction validation and ingestion.
- Manage investigation cases and investigator assignments.
- Apply authentication and role-based access control.
- Explore AI-assisted financial crime investigation capabilities.

## Technology Stack

### Backend
- Rust
- Axum
- Tokio
- SQLx
- PostgreSQL
- JWT authentication

### Frontend
- React
- TypeScript
- Vite

## Implemented Features

- User registration and login.
- Password hashing.
- JWT authentication.
- Role-based authorization policies.
- Case management endpoints.
- Investigator model and case assignment.
- Financial transaction domain model.
- PostgreSQL transaction storage.
- Transaction ingestion API.
- Transaction validation and normalization.
- CSV bulk transaction ingestion.

*Implementation status should be verified against the current code before each release.*

## Security

The backend includes:

- JWT-based authentication.
- Role-based access control.
- Password hashing.
- Request validation.
- Database constraints.
- Restricted transaction ingestion permissions.

Transaction ingestion is restricted to senior investigators and administrators.

Public registration assigns the ordinary investigator role.

## Project Structure

```text
FinSightAI/
├── backend/
│   ├── migrations/
│   ├── src/
│   │   ├── auth/
│   │   ├── models/
│   │   ├── repositories/
│   │   ├── services/
│   │   └── routes.rs
│   ├── Cargo.toml
│   └── Cargo.lock
├── frontend/
└── README.md
```

## Development Setup

### Requirements

- Rust toolchain
- PostgreSQL
- Cargo

### Backend

```bash
cd backend
cp .env.example .env
```

Configure your PostgreSQL connection and JWT secret in `.env`.

Run database migrations:

```bash
sqlx migrate run
```

Start the backend:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

**Note:** The frontend setup and API documentation will be expanded as those components are developed.

## Roadmap

The project is being developed incrementally, with milestones covering:

1. Core case management.
2. Authentication and authorization.
3. Investigator management.
4. Financial transaction ingestion.
5. Transaction validation and bulk CSV ingestion.
6. Transaction-to-case linking.
7. Investigation evidence and workflows.
8. AI-assisted investigation capabilities.

## Disclaimer

FinSight AI is an educational and portfolio software engineering project under active development. It is not a certified AML compliance solution and should not be used for real financial crime investigations without appropriate security, compliance, and operational validation.

## Author

**Metempsychosis16**

GitHub: https://github.com/Metempsychosis16
