FROM rust:slim AS builder

WORKDIR /app

# Copy manifests first so dependency compilation can be cached.
COPY Cargo.toml Cargo.lock ./

# Copy source and migrations required at compile time.
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install CA certificates for TLS connections if needed.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Run the application as a non-root user.
RUN useradd --system --no-create-home --shell /usr/sbin/nologin appuser

COPY --from=builder /app/target/release/rust-lab /usr/local/bin/rust-lab

USER appuser

EXPOSE 3000

CMD ["rust-lab"]