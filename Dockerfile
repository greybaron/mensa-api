FROM rust:bookworm AS build
COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --release

FROM debian:bookworm-slim AS mensa-api
COPY --from=build ./target/release/mensi-api /app/mensa-api
WORKDIR /app/data
EXPOSE 8080
ENTRYPOINT ["/app/mensa-api"]