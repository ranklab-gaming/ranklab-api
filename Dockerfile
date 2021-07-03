FROM rust:latest

WORKDIR /usr/src
RUN cargo new --bin ranklab-api

WORKDIR /usr/src/ranklab-api
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

ADD . .
RUN cargo build --release

FROM debian:stable-slim

WORKDIR /root/app
RUN apt-get update && apt-get install -y openssl
COPY --from=0 /usr/src/ranklab-api/target/release/ranklab-api .
COPY Rocket.toml .

EXPOSE 8000
ENTRYPOINT ["/root/app/ranklab-api"]
