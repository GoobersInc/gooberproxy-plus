FROM rustlang/rust:nightly AS builder

WORKDIR /build

COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./.gitmodules ./.gitmodules
COPY ./.git ./.git

# Clone the git submodules
RUN git submodule update --init --recursive

# Build your program for release
RUN cargo build --release

# Now make a new image with just the binary, using alpine
FROM alpine:latest

WORKDIR /app

# Copy the binary from the builder image
COPY --from=builder /build/target/release/gooberproxy-plus .

VOLUME /app/config/config.toml

# Run the binary
ENTRYPOINT ["./gooberproxy-plus --config_path /app/config/config.toml"]