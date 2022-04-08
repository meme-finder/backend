FROM rust:1.59

WORKDIR /usr/src/back
COPY . .

RUN cargo build --release

CMD ["./target/release/back"]

FROM rust:1.59 as build

# create a new empty shell project
RUN USER=root cargo new --bin back
WORKDIR /back

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY . .

# build for release
RUN cargo build --release

# our final base
FROM rust:1.59-slim-buster

# copy the build artifact from the build stage
COPY --from=build /back/target/release/back .

# set the startup command to run your binary
CMD [ "./back" ]