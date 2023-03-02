FROM rust:alpine as builder

RUN apk add musl-dev build-base cmake

WORKDIR /builder

RUN cargo new --bin app && \
    cargo new --lib app/entity && \
    cargo new --lib app/migration && \
    cargo new --lib app/macros && \
    cargo new --lib app/cmds

WORKDIR /builder/app

COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ./entity/Cargo.toml ./entity/Cargo.toml
COPY ./macros/Cargo.toml ./macros/Cargo.toml

RUN rm ./macros/src/lib.rs && \
    touch ./macros/src/lib.rs && \
    cargo build --release && \
    rm -rf ./src/ \
           ./entity/src/ \
	   ./macros/src/

COPY ./src/ ./src/
COPY ./entity/src/ ./entity/src/
COPY ./macros/src/ ./macros/src/

RUN rm target/release/deps/matverseny_backend* \
       target/release/deps/entity* \
       target/release/deps/libentity* \
       target/release/deps/macros* \
       target/release/deps/libmacros* && \
    cargo build --release

FROM alpine
WORKDIR /app
COPY --from=builder /builder/app/target/release/matverseny-backend ./
EXPOSE 3002

RUN addgroup -S matverseny && \
    adduser -S -D -H -s /bin/false -G matverseny matverseny && \
    chown -R matverseny:matverseny /app
USER matverseny

CMD ["./matverseny-backend"]


