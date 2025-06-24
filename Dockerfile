FROM rust:bookworm AS builder

COPY --from=golang:1.24-alpine /usr/local/go /usr/local/go

ENV PATH=$PATH:/usr/local/go/bin

RUN apt update && apt install -y clang && rm -rf /var/lib/apt/lists

WORKDIR /src
COPY . .

RUN --mount=type=cache,target=/go/pkg/mod \ 
    cd libwg/wireguard-go && \
    CGO_ENABLED=1 go build -trimpath -v -buildmode=c-archive ./libwg && \
    mv libwg.a libwg.h ..

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    cargo build --release && \
    cp target/release/corplink-rs /


FROM debian:bookworm-slim AS corplink

COPY --from=builder /corplink-rs /corplink-rs

ENTRYPOINT [ "/corplink-rs" ]
