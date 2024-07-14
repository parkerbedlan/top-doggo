# https://github.com/kpcyrd/mini-docker-rust/blob/main/Dockerfile
# FROM rust:latest
# FROM rustlang/rust:nightly
# FROM rust:1.77.2
FROM rust:alpine3.20 AS builder
WORKDIR /app

# RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static build-base protoc pkgconfig
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

# for nonalpine, but probably generally unnecessary
# RUN apt-get update && apt-get install -y sqlite3 libsqlite3-dev
# RUN sqlite3 --version

COPY Cargo.* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

# if you wanna builds the schema file in the image instead of copying it over from host machine
RUN mkdir .cargo
RUN cargo install sqlx-cli --root .cargo
COPY .env ./
COPY ./migrations/ ./migrations/
RUN mkdir db
RUN .cargo/bin/sqlx db create
RUN .cargo/bin/sqlx migrate run
RUN rm -rf .cargo

COPY . .
# if you copied the schema file from the host machine instead of building it
# RUN mv db/schema.db db/todos.db

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release

RUN rm -rf db

# use a plain alpine image, the alpine version needs to match the builder
FROM alpine:3.20
# if needed, install additional dependencies here
RUN apk add --no-cache libgcc
# copy the binary into the final image
# --from=builder
COPY --from=builder /app/target/release/best-doggo .
COPY --from=builder /app/.env .
COPY --from=builder /app/assets/ ./assets/
EXPOSE 3000
# set the binary as entrypoint
CMD ["./best-doggo"]
# ENTRYPOINT ["/best-doggo"]
