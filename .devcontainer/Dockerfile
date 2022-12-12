FROM ghcr.io/ranklab-gaming/ranklab-devcontainer:v1.3.3

ENV CARGO_HOME /home/vscode/.cargo
ENV RUSTUP_HOME /home/vscode/.rustup
ENV PATH="${PATH}:/home/vscode/.cargo/bin"

RUN sudo apt-get update && sudo DEBIAN_FRONTEND=noninteractive apt-get upgrade -y
RUN sudo apt-get install -y libssl-dev libpq-dev clang jq
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2022-12-05

RUN version=$(wget -q -O- https://api.github.com/repos/rui314/mold/releases/latest | jq -r .tag_name | sed 's/^v//'); \
  wget -q -O- https://github.com/rui314/mold/releases/download/v$version/mold-$version-$(uname -m)-linux.tar.gz | \
  sudo tar -C /usr/local --strip-components=1 -xzf -

RUN sudo ln -sf /usr/local/bin/mold $(realpath /usr/bin/ld)
RUN cargo install diesel_cli@2.0.0-rc.1 --no-default-features --features postgres
RUN cargo install cargo-watch
