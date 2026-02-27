FROM rust:1.93-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static build-base pkgconfig upx
COPY ../../Cargo.toml Cargo.lock ./

RUN cargo fetch --locked

COPY ../../src ./src

RUN MUSL_TARGET="$(rustc -vV | sed -n 's/^host: //p')" && \
    CARGO_INCREMENTAL=0 \
    RUSTFLAGS="-C strip=symbols" \
    cargo build --release --locked --target "${MUSL_TARGET}" && \
    file "target/${MUSL_TARGET}/release/ettac" | grep -qi "statically linked" && \
    upx --best --lzma "target/${MUSL_TARGET}/release/ettac"

FROM scratch

COPY --from=builder /app/target/*-musl/release/ettac /ettac

ENTRYPOINT ["/ettac"]
