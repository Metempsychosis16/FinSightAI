## FinSight AI — Production Architecture

## 1. Objective

Define the production architecture for FinSight AI, including application
components, network boundaries, security, data protection, observability,
and deployment requirements.

This document describes the intended architecture. It does not claim that
cloud infrastructure has already been deployed.

## 2. Application Architecture

The production request flow is:

Browser
  |
  | HTTPS
  v
Reverse Proxy (Nginx)
  |
  +---- Static frontend assets
  |       |
  |       v
  |     React + TypeScript
  |
  +---- /api requests
          |
          v
       Rust + Axum API
          |
          v
       PostgreSQL

The React frontend provides the user interface.
The Rust/Axum backend handles authentication, authorization, business logic,
transaction analysis, and API requests.
PostgreSQL stores application data.

## 3. Network Architecture

- Public traffic enters through the HTTPS reverse proxy.
- The frontend is served through Nginx.
- API requests are forwarded to the Axum backend.
- The backend communicates with PostgreSQL over a private network.
- PostgreSQL must not be directly exposed to the public internet.
- Internal services communicate using their Docker Compose service names.
- The backend requires outbound HTTPS access to the configured AI provider.
- AI provider credentials must remain server-side.


## 4. Security

- Use HTTPS for production traffic.
- Keep JWT secrets and database credentials outside source control.
- Enforce authentication and role-based authorization on protected endpoints.
- Validate incoming API data.
- Do not expose internal service ports unnecessarily.
- Return safe error messages to clients.
- Restrict database access to the backend service.

## 5. Data Protection

- Store database credentials in environment configuration or a secrets manager.
- Never commit `.env` files or production secrets.
- Back up PostgreSQL data regularly.
- Restrict access to database backups.
- Define a recovery procedure before production deployment.

## 6. Observability

- Use structured application logging through tracing.
- Provide a liveness endpoint to verify that the backend is running.
- Provide a database health endpoint to verify database connectivity.
- Use container health checks to monitor service health.
- Avoid logging passwords, JWTs, API keys, or sensitive transaction data.

## 7. Deployment Strategy

- Initially run the containerized frontend, Nginx reverse proxy,
  and Axum backend on one cloud compute instance.
- Use managed PostgreSQL for persistent application data.
- Keep the database reachable only through a private network.
- Expose the application through an HTTPS entry point.
- Provision cloud infrastructure .
- Deploy and verify the live application .
- TLS certificates must be provisioned and renewed through a defined process.
- HTTP traffic should be redirected to HTTPS.

Cloud infrastructure has not yet been provisioned.
This document describes the intended architecture.



## 8. Production Requirements

- React frontend builds successfully.
- Rust backend builds successfully.
- API requests pass through the reverse proxy.
- PostgreSQL is reachable only through the intended private network.
- Secrets are excluded from Git.
- Health checks report service status.
- Logs support troubleshooting without exposing sensitive information.
- Database backups and recovery procedures are defined.
- The backend requires outbound HTTPS access to the configured AI provider.
- AI provider credentials must remain server-side.
