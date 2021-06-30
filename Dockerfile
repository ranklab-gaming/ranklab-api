FROM rust:latest as build

ENV RUSTFLAGS="-C target-feature=+crt-static"
ENV APP="ranklab-api"
ENV TARGET="x86_64-unknown-linux-musl"

RUN apt-get update && apt-get -y install musl-tools
RUN rustup target add ${TARGET}

WORKDIR /usr/src
RUN cargo new --bin ${APP}
WORKDIR /usr/src/${APP}
COPY ./Cargo.toml .
RUN cargo build --target ${TARGET} --release --features vendored
RUN rm ./src/*
RUN rm ./target/${TARGET}/release/${APP} && rm ./target/${TARGET}/release/${APP}.*
ADD . ./
RUN cargo build --target ${TARGET} --release --features vendored

FROM alpine:latest

ENV APP="ranklab-api"
ENV TARGET="x86_64-unknown-linux-musl"
WORKDIR /app

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

COPY --from=build /usr/src/${APP}/target/${TARGET}/release/${APP} .
COPY --from=build /usr/src/${APP}/Rocket.toml .

EXPOSE 8000
ENTRYPOINT ["/app/ranklab-api"]
