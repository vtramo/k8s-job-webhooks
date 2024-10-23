FROM lukemathwalker/cargo-chef:latest as chef
WORKDIR /app

FROM chef AS planner
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
RUN cargo chef prepare

FROM chef AS builder
COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release
COPY ./migrations ./migrations
COPY ./queries ./queries
COPY ./Cargo.toml ./Cargo.lock ./
COPY ./src ./src
ENV DATABASE_URL="sqlite://./sqlite.db"
RUN cargo install sqlx-cli
RUN sqlx db create
RUN sqlx migrate run --source ./migrations/sqlite/

RUN cargo build --release
RUN mv ./target/release/k8s-job-webhooks ./app

FROM debian:stable-slim AS runtime
WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/
COPY --from=builder /app/sqlite.db ./sqlite.db
ENV JOB_FAMILY_WATCHERS_CONFIG_FILE="job-family-config.yaml"
ENV DATABASE_URL="sqlite://./sqlite.db"
ENTRYPOINT ["/usr/local/bin/app"]