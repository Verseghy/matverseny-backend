FROM registry.access.redhat.com/ubi9/ubi as chef

RUN curl --proto '=https' --tlsv1.3 -sSf https://sh.rustup.rs > rustup-init.sh && \
    sh rustup-init.sh --default-toolchain "1.79" --profile minimal -y && \
    source "$HOME/.bashrc" && \
    dnf install clang -y

ENV PATH="$PATH:/root/.cargo/bin"

RUN cargo install cargo-chef --locked --version "0.1.67" && \
    rm -rf $CARGO_HOME/registry/



FROM chef AS planner
WORKDIR /planner
COPY . .
RUN cargo chef prepare --bin setup --recipe-path recipe.json



FROM chef AS builder
WORKDIR /builder
COPY --from=planner /planner/recipe.json recipe.json
# Build dependencies
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --bin setup --release



FROM registry.access.redhat.com/ubi9/ubi-micro
WORKDIR /app
COPY --from=builder /builder/target/release/setup ./
CMD ["./setup"]
