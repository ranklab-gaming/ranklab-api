FROM rust:alpine AS builder

RUN apk add --update-cache \
    openssl-dev \
    postgresql-dev \
    musl-dev \
    clang \
    && rm -rf /var/cache/apk/*

WORKDIR /usr/src/
RUN cargo new ranklab-api
WORKDIR /usr/src/ranklab-api
COPY rust-toolchain Cargo.toml Cargo.lock ./
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

FROM alpine:latest

RUN apk add --update-cache \
    libgcc \
    libpq \
    && rm -rf /var/cache/apk/*

WORKDIR /root/app
COPY diesel.toml Ranklab.toml Rocket.toml ./
COPY --from=builder /usr/src/ranklab-api/target/release/ranklab-api ./
EXPOSE 8000
ENTRYPOINT ["/root/app/ranklab-api"]
