FROM rust:alpine AS builder

RUN apk add --update-cache \
    openssl-dev \
    postgresql-dev \
    musl-dev \
    && rm -rf /var/cache/apk/*

WORKDIR /usr/src/
RUN cargo new ranklab-api
WORKDIR /usr/src/ranklab-api
COPY rust-toolchain Cargo.toml Cargo.lock ./
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release && rm -rf $HOME/.cargo/{git,registry}
COPY src .
RUN cargo install --path . && rm -rf $HOME/.cargo/{git,registry}

FROM alpine:latest

WORKDIR /root/app
COPY diesel.toml Ranklab.toml Rocket.toml ./
COPY --from=builder /usr/local/cargo/bin/ranklab-api ./
EXPOSE 8000
ENTRYPOINT ["/root/app/ranklab-api"]
