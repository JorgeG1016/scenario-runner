FROM rust:slim-trixie as build_stage

# Set up dependencies
RUN apt-get update
RUN apt-get install -y libudev-dev pkg-config

# Set up app environment
WORKDIR /app
COPY . .
RUN rustup component add clippy

# Lint and run unit tests
RUN cargo clippy --all-targets --all-features -- -D warnings
RUN cargo test --all

# Build the final release
RUN cargo build --release

FROM debian:trixie-slim

# Set up runtime dependencies
RUN apt-get install -y libudev1

# Grab the output from the build stage
COPY --from=build_stage /app/target/release/scenario-runner /usr/local/bin/scenario-runner

# Need to have the app run when "docker run" is used
ENTRYPOINT [ "/usr/local/bin/scenario-runner" ]
