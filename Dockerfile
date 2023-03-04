FROM rustlang/rust:nightly AS builder

WORKDIR /build

RUN apt-get update && \
    apt-get dist-upgrade -y && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src

RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

WORKDIR /app

# Copy the binary from the builder image
COPY --from=builder /app/target/gooberproxy-plus .

VOLUME /app/config/config.toml

# Run the binary
ENTRYPOINT ["/app/target/gooberproxy-plus --config_path /app/config/config.toml"]