# Temporary build container
FROM rust:1-alpine AS builder

# Get ENV variables for build info from build args
ARG GITHUB_SHA="n/a"
ARG GITHUB_REF_NAME="n/a"
ENV GITHUB_SHA=$GITHUB_SHA
ENV GITHUB_REF_NAME=$GITHUB_REF_NAME

# Install build dependencies
RUN apk add --no-cache musl-dev make cmake g++

# Copy source code into container
WORKDIR /usr/src
COPY . .

# Build Rust binary
ENV CARGO_TARGET_DIR=/usr/src/target
RUN cargo build --release

# Remove debug symbols
RUN strip /usr/src/target/release/dmarc-report-viewer

# Build final minimal image with only the binary
FROM scratch
COPY --from=builder /usr/src/target/release/dmarc-report-viewer /
EXPOSE 8080
HEALTHCHECK --start-period=30s CMD ["./dmarc-report-viewer", "--health-check"]
CMD ["./dmarc-report-viewer"]
