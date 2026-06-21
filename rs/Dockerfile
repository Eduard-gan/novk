# cargo-chef keeps dependency compilation in a reusable layer.
FROM rust:1-slim-trixie AS chef
RUN cargo install cargo-chef
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin novk-cli

FROM debian:trixie-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m -u 1000 appuser
WORKDIR /app

COPY --from=builder /build/target/release/novk-cli .
COPY --from=builder /build/config ./config
COPY --from=builder /build/assets ./assets

RUN mkdir -p /data /uploaded \
    && chown -R appuser:appuser /app /data /uploaded

USER appuser
ENV LOCO_ENV=production
EXPOSE 5150
CMD ["./novk-cli", "start"]
