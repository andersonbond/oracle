# syntax=docker/dockerfile:1
# Build from the Oracle source root, with this file passed through -f.
# These maintained tags are convenient defaults. For a controlled release,
# supply reviewed name@sha256:... references and record the resulting image.
ARG RUST_IMAGE=rust:1-bookworm
ARG RUNTIME_IMAGE=debian:bookworm-slim

FROM ${RUST_IMAGE} AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY skills/ ./skills/
RUN cargo build --release --locked --bin oracle

FROM ${RUNTIME_IMAGE} AS runtime
# Oracle invokes Git for repository inspection. Keep that runtime dependency;
# a smaller image that silently loses inspection capability is not equivalent.
RUN apt-get update \
    && apt-get install -y --no-install-recommends git ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir /workspace \
    && chown 10001:10001 /workspace
COPY --from=build /build/target/release/oracle /usr/local/bin/oracle
USER 10001:10001
WORKDIR /workspace
# Exec form preserves the CLI's arguments and exit status. This is a one-shot
# tool, so there is no listening port, restart loop, or service health probe.
ENTRYPOINT ["/usr/local/bin/oracle"]
CMD ["--help"]
