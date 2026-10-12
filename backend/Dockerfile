# syntax=docker/dockerfile:1
FROM rust:1.99.0-slim-trixie AS build
WORKDIR /src
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release -p vd-api && cp target/release/vd-api /vd-api

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* && useradd -r -u 10001 app
COPY --from=build /vd-api /usr/local/bin/vd-api
USER app
EXPOSE 8080
CMD ["vd-api"]
