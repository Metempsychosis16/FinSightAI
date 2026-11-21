
# Stage 1: Build the Rust application
FROM rust:1.88-bookworm AS builder

WORKDIR /app

# Copy dependency manifests first
COPY Cargo.toml Cargo.lock ./

# Copy application source
COPY src ./src
COPY migrations ./migrations

# Compile the release executable
ENV CARGO_BUILD_JOBS=1
RUN cargo build --release --locked

# Stage 2: Minimal runtime image
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Required for HTTPS requests made by reqwest
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
       ca-certificates \
       curl \
    && rm -rf /var/lib/apt/lists/*


# Copy only the compiled executable
COPY --from=builder /app/target/release/backend /usr/local/bin/backend

EXPOSE 3000

HEALTHCHECK --interval=30s \
  --timeout=5s \
  --start-period=20s \
  --retries=3 \
  CMD curl --fail --silent http://127.0.0.1:3000/health/database || exit 1


CMD ["backend"]

