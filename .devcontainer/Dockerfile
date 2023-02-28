FROM ghcr.io/ranklab-gaming/ranklab-devcontainer:v2.0.0

ENV CARGO_HOME=/home/vscode/.cargo
ENV RUSTUP_HOME=/home/vscode/.rustup
ENV PATH="${PATH}:/home/vscode/.cargo/bin"

RUN export DEBIAN_FRONTEND=noninteractive \
    && sudo apt-get update \
    && sudo apt-get upgrade -y \
    && sudo apt-get install -y libssl-dev libpq-dev clang jq

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2023-02-28
RUN gh release download -p "*-$(uname -m)-linux.tar.gz" -R rui314/mold -O - | sudo tar -C /usr/local --strip-components=1 -xzf -
RUN sudo ln -sf /usr/local/bin/mold $(realpath /usr/bin/ld)
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
