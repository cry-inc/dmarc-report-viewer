FROM rust:1-alpine as builder
RUN apk add --no-cache musl-dev make cmake g++
WORKDIR /usr/src
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl --release
RUN strip /usr/src/target/x86_64-unknown-linux-musl/release/dmarc-report-viewer

FROM scratch
COPY --from=builder /usr/src/target/x86_64-unknown-linux-musl/release/dmarc-report-viewer /
CMD ["./dmarc-report-viewer"]
