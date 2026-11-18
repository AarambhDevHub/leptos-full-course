FROM rust:1-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --locked
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo leptos build --release -p module-25-production

FROM debian:bookworm-slim AS runtime
# curl is only here for the container health check below.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --no-create-home app
WORKDIR /app
COPY --from=builder /app/target/release/module-25-production /app/module-25-production
COPY --from=builder /app/module-25-production/target/site /app/target/site
# cargo-leptos reads Cargo.toml for these when it runs the server for you.
# A container runs the bare binary, so without them the page would ask the
# browser for /pkg/.js and the app would never hydrate.
ENV LEPTOS_OUTPUT_NAME=module-25-production \
    LEPTOS_SITE_ROOT=target/site \
    LEPTOS_SITE_PKG_DIR=pkg \
    LEPTOS_SITE_ADDR=0.0.0.0:3000 \
    RUST_LOG=info
# JWT_SECRET and DATABASE_URL are deliberately not set here: secrets do not
# belong in an image. The server refuses to start without a real JWT_SECRET.
USER app
EXPOSE 3000
HEALTHCHECK --interval=30s --timeout=3s --start-period=20s --retries=3 \
    CMD curl -fsS http://127.0.0.1:3000/health || exit 1
CMD ["/app/module-25-production"]
