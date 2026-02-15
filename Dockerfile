FROM rust:alpine3.23 AS builder
RUN mkdir -p /src
COPY . /src
WORKDIR /src
RUN cargo build --release

FROM alpine:3.23
RUN mkdir -p /app
COPY --from=builder ./src/target/release/simd-detective /usr/bin/simd-detective
COPY --from=builder ./src/data/simd-detective-intrinsics.json /usr/share/simd-detective-intrinsics.json
