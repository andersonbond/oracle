# syntax=docker/dockerfile:1
# Build context is this example directory, not the Oracle source root.
ARG RUST_IMAGE=rust:1-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim
FROM ${RUST_IMAGE} AS build
WORKDIR /example
COPY Cargo.toml.example Cargo.toml
COPY Cargo.lock.example Cargo.lock
COPY main.rs ./
RUN cargo test --locked && cargo build --release --locked

FROM ${RUNTIME_IMAGE} AS runtime
COPY --from=build /example/target/release/oracle-cache-example /usr/local/bin/oracle-cache-example
USER 10001:10001
ENTRYPOINT ["/usr/local/bin/oracle-cache-example"]
CMD ["verify"]
