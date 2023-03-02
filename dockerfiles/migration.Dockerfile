FROM rust:alpine as builder

RUN apk add musl-dev

WORKDIR /builder

RUN cargo new --bin app && \
    cargo new --lib app/entity && \
    cargo new --lib app/migration && \
    cargo new --lib app/macros && \
    cargo new --lib app/cmds

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

FROM alpine
WORKDIR /app
COPY --from=builder /builder/app/target/release/migration ./

RUN addgroup -S matverseny && \
    adduser -S -D -H -s /bin/false -G matverseny matverseny && \
    chown -R matverseny:matverseny /app
USER matverseny

CMD ["./migration"]
