FROM rust:bookworm AS build
COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --release

FROM debian:bookworm-slim AS mensa-api
RUN apt-get update && \
  apt-get install -y \
  libssl3 \
  libsqlite3-0 \
  ca-certificates \
  && \
  apt-get autoremove -y && \
  apt-get clean -y && \
  rm -rf /var/lib/apt/lists/*
RUN c_rehash
COPY --from=build ./target/release/mensa-api /app/mensa-api
WORKDIR /app/data
EXPOSE 8080
ENTRYPOINT ["/app/mensa-api"]