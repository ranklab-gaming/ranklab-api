FROM rust:latest

ARG USER_UID=1000
ARG USER_GID=$USER_UID

ENV WORKSPACE /workspace

RUN apt-get update -qq \
  && apt-get install -qq --no-install-recommends lsb-release \
  && echo "deb http://apt.postgresql.org/pub/repos/apt/ $(lsb_release -cs)-pgdg main" > /etc/apt/sources.list.d/pgdg.list \
  && curl --silent https://www.postgresql.org/media/keys/ACCC4CF8.asc | apt-key add - \
  && apt-get update -qq \
  && apt-get install -qq --no-install-recommends postgresql-client-13 gnupg2 \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid $USER_GID vscode \
  && useradd --uid $USER_UID --gid $USER_GID -m vscode -s /bin/bash

USER $USER_UID
WORKDIR $WORKSPACE

RUN cargo install diesel_cli --no-default-features --features postgres
