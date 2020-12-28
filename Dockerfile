FROM rust:1.48.0 as build
# create empty project
WORKDIR /usr/src
RUN USER=root cargo new --bin app
WORKDIR /usr/src/app
# copy manifest
COPY Cargo.lock .
COPY Cargo.toml .
# build dependencies
RUN cargo build --release
RUN rm -rf src/
# copy source
COPY src/ src/
# build release
RUN cargo build --release

FROM rust:1.48.0
COPY --from=build /usr/src/app/target/release/rust-kata-001 .
CMD ["./rust-kata-001"]
