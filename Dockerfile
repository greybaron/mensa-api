FROM rust:bookworm AS build
COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --release

FROM debian:bookworm-slim AS campus-api
RUN apt-get update && \
  apt-get install -y \
  libssl3 \
  ca-certificates \
  && \
  apt-get autoremove -y && \
  apt-get clean -y && \
  rm -rf /var/lib/apt/lists/*
RUN c_rehash
COPY --from=build ./target/release/campus-api /app/campus-api
WORKDIR /app/data
EXPOSE 8080
ENTRYPOINT ["/app/campus-api"]