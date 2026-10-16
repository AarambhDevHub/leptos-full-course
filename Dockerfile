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
WORKDIR /app
ENV RUST_LOG=info
COPY --from=builder /app/target/release/module-25-production /app/module-25-production
COPY --from=builder /app/module-25-production/target/site /app/target/site
EXPOSE 3000
CMD ["/app/module-25-production"]
