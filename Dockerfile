FROM rust:1.88.0-alpine3.22 as build_stage

# Set up dependencies
RUN apk update
RUN apk add \
    build-base \
    libudev-zero-dev \
    pkgconf

# Set up app environment
WORKDIR /app
COPY . .
RUN rustup component add clippy

# Lint and run unit tests
RUN cargo clippy --all-targets --all-features -- -D warnings
RUN cargo test --all

# Build the final release
RUN cargo build --release

FROM alpine:3.22

# Set up runtime dependencies
RUN apk add eudev

# Grab the output from the build stage
COPY --from=build_stage /app/target/release/scenario-runner /usr/local/bin/scenario-runner

# Need to have the app run when "docker run" is used
ENTRYPOINT [ "/usr/local/bin/scenario-runner" ]
