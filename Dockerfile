FROM rust:latest

RUN apt-get update \
    && apt-get install -y libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src
RUN cargo new --bin ranklab-api

WORKDIR /usr/src/ranklab-api
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

ADD . .
RUN cargo build --release

FROM debian:stable-slim

WORKDIR /root/app
RUN apt-get update && apt-get install -y gnupg wget lsb-release
RUN echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list
RUN wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add -
RUN apt-get update && apt-get install -y openssl postgresql-client && rm -rf /var/lib/apt/lists/*
COPY --from=0 /usr/src/ranklab-api/target/release/ranklab-api .
COPY Rocket.toml Ranklab.toml ./

EXPOSE 8000
ENTRYPOINT ["/root/app/ranklab-api"]
