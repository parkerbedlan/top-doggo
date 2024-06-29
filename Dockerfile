# https://github.com/kpcyrd/mini-docker-rust/blob/main/Dockerfile
# FROM rust:latest
# FROM rustlang/rust:nightly
# FROM rust:1.77.2
FROM rust:alpine3.20
WORKDIR /usr/src/best-doggo

RUN apk add --no-cache musl-dev

# for nonalpine, but probably generally unnecessary
# RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev
# RUN sqlite3 --version

COPY Cargo.* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# if you wanna build the schema file in the image instead of copying it over from host machine. uses slightly more space but less steps to worry about.
# RUN mkdir .cargo
# RUN cargo install sqlx-cli --root .cargo
# COPY .env ./
# COPY ./migrations/ ./migrations/
# RUN mkdir db
# RUN .cargo/bin/sqlx db create
# RUN .cargo/bin/sqlx migrate run
# RUN rm -rf .cargo

COPY . .
RUN mv db/schema.db db/todos.db

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release

RUN rm -rf db

EXPOSE 3000

CMD ["./target/release/best-doggo"]
