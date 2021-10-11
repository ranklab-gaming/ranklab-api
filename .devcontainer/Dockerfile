FROM ghcr.io/ranklab-gaming/ranklab-devcontainer:latest

ENV CARGO_HOME /home/vscode/.cargo
ENV RUSTUP_HOME /home/vscode/.rustup
ENV PATH="${PATH}:/home/vscode/.cargo/bin"

RUN echo "deb http://apt.postgresql.org/pub/repos/apt/ $(lsb_release -cs)-pgdg main" | sudo tee /etc/apt/sources.list.d/pgdg.list \
  && curl --silent https://www.postgresql.org/media/keys/ACCC4CF8.asc | sudo apt-key add - \
  && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly \
  && sudo apt-get update -qq \
  && sudo apt-get install -qq --no-install-recommends postgresql-client-13 build-essential libssl-dev pkg-config libpq-dev \
  && sudo apt-get clean \
  && sudo rm -rf /var/lib/apt/lists/*

RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch