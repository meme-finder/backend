FROM rust:1.60 as build

# create a new empty shell project
RUN USER=root cargo new --bin backend
WORKDIR /backend

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release && rm ./src/*.rs ./target/release/deps/backend*

# copy your source tree
ADD . ./

# build for release
RUN cargo build --release

# our final base
FROM debian:11-slim

# Create new user
RUN useradd --create-home app
WORKDIR /home/app

# Healthcheck
HEALTHCHECK --interval=5s --timeout=10s --retries=3 CMD curl -sS 127.0.0.1:8080/health || exit 1
RUN apt-get update && \
    apt-get install --no-install-recommends -y curl && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# Switch user
USER app

# copy the build artifact from the build stage
COPY --from=build /backend/target/release/backend /usr/local/bin

# set the startup command to run your binary
CMD [ "/usr/local/bin/backend" ]
