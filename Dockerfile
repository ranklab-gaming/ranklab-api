FROM rust:latest

RUN apt-get update \
    && apt-get install -y gnupg wget lsb-release libpq-dev lld clang \
    && echo "deb http://apt.postgresql.org/pub/repos/apt $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
    && wget --quiet -O - https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - \
    && apt-get update \
    && apt-get install -y openssl postgresql-client \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/ranklab-api
ADD . .

RUN cargo build --release \
  && mkdir -p /root/app \
  && cp target/release/ranklab-api Rocket.toml Ranklab.toml /root/app \
  && rm -rf /usr/src/ranklab-api

WORKDIR /root/app
EXPOSE 8000
ENTRYPOINT ["/root/app/ranklab-api"]
