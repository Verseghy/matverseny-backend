FROM registry.access.redhat.com/ubi9/ubi as builder

WORKDIR /builder

RUN curl --proto '=https' --tlsv1.3 -sSf https://sh.rustup.rs > rustup-init.sh && \
    sh rustup-init.sh --default-toolchain 1.74 --profile minimal -y && \
    source "$HOME/.bashrc" && \
    cargo new --bin app && \
    cargo new --lib app/entity && \
    cargo new --lib app/migration && \
    cargo new --lib app/macros && \
    cargo new --lib app/cmds && \
    dnf install clang cmake -y

ENV PATH="$PATH:/root/.cargo/bin"

WORKDIR /builder/app

COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ./migration/Cargo.toml ./migration/Cargo.toml
COPY ./entity/Cargo.toml ./entity/Cargo.toml

RUN rm ./macros/src/lib.rs && \
    touch ./macros/src/lib.rs && \
    cargo build -p migration --release && \
    rm -rf ./migration/src \
           ./entity/src/

COPY ./migration/src/ ./migration/src/
COPY ./entity/src/ ./entity/src/

RUN rm target/release/deps/migration* \
       target/release/deps/entity* \
       target/release/deps/libentity* && \
    cargo build -p migration --release





FROM registry.access.redhat.com/ubi9/ubi-micro

WORKDIR /app
COPY --from=builder /builder/app/target/release/migration ./

CMD ["./migration"]
