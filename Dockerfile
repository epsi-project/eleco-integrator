FROM lukemathwalker/cargo-chef:latest-rust-1.64.0 as chef
WORKDIR /app
RUN apt update && apt install lld clang -y

FROM chef as planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef as builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin portalz-integrator

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/eleco-integrator eleco-integrator
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENV RUST_LOG info
ENV APP_RABBITMQ_HOST $APP_RABBITMQ_HOST
ENV APP_RABBITMQ_PORT $APP_RABBITMQ_PORT
ENV APP_RABBITMQ_AUTH_USERNAME $APP_RABBITMQ_AUTH_USERNAME
ENV APP_RABBITMQ_AUTH_PASSWORD $APP_RABBITMQ_AUTH_PASSWORD
ENTRYPOINT ["./eleco_integrator"]