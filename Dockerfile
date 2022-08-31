# 1. BUILD-CONTAINER: Frontend
FROM node:gallium-alpine

WORKDIR /root

COPY ./frontend /root

RUN yarn && yarn build

# 2. BUILD-CONTAINER: Server
FROM rust:alpine

WORKDIR /root

RUN apk add --no-cache musl-dev

COPY ./bin /root/bin
COPY ./lib /root/lib
COPY ./server /root/server
COPY ./Cargo.toml /root/Cargo.toml
COPY ./Cargo.lock /root/Cargo.lock

RUN cargo build --workspace --release

# 3. RUNTIME-CONTAINER: run server with frontend as assets
FROM alpine:latest

WORKDIR /root

COPY --from=0 /root/dist /root/assets
COPY --from=1 /root/target/release/adf-bdd-server /root/server

ENTRYPOINT ["./server"]

