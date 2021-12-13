FROM ghcr.io/ranklab-gaming/ranklab-devcontainer:v1.2.15

ENV CARGO_HOME /home/vscode/.cargo
ENV RUSTUP_HOME /home/vscode/.rustup
ENV PATH="${PATH}:/home/vscode/.cargo/bin"

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2021-12-05
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
