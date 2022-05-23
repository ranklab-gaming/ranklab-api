FROM ghcr.io/ranklab-gaming/ranklab-devcontainer:v1.3.2

ENV CARGO_HOME /home/vscode/.cargo
ENV RUSTUP_HOME /home/vscode/.rustup
ENV PATH="${PATH}:/home/vscode/.cargo/bin"

RUN sudo apt-get install -y libssl-dev libpq-dev
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2022-05-22
RUN cargo install diesel_cli --no-default-features --features postgres
RUN cargo install cargo-watch
