# --- Stage 1: Build Rust backend ---
FROM rust:1.91-bookworm AS backend-builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/

ENV SQLX_OFFLINE=true
RUN cargo build --release

# --- Stage 2: Build frontend ---
FROM node:22-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# --- Stage 3: Runtime ---
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=backend-builder /app/target/release/knowledgebase-agent ./
COPY --from=frontend-builder /app/frontend/dist ./frontend/dist/
COPY migrations/ migrations/

EXPOSE 3000
CMD ["./knowledgebase-agent"]
