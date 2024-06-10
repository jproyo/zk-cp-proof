# Multistage build to reduce image size
# First stage builds the binary
FROM rust:1.78-slim-bookworm as base

RUN apt update && \
    apt install -y make protobuf-compiler iproute2 iputils-ping iperf net-tools dnsutils ssh git gcc libssl-dev libprotobuf-dev

WORKDIR /app/zk-material
COPY . .

# Install the binary leaving it in the WORKDIR/bin folder
RUN mkdir -p /app/zk-material/bin
RUN cargo install --path protocol --root protocol --bins

# Second stage builds the runtime image.
# This stage will be the final image
FROM debian:stable-slim as go-runtime

RUN apt update && \
    apt install -y iproute2 iputils-ping iperf net-tools dnsutils libssl-dev libprotobuf-dev curl netcat-openbsd
WORKDIR /app/zk-material
RUN mkdir -p /app/zk-material/config

# Set the path to include the binaries and not just the default /usr/local/bin
ENV PATH="$PATH:/app/zk-material/bin"

# Third stage: Copy the binaries from the base stage and the go-runtime stage
FROM debian:stable-slim as runtime
WORKDIR /app/zk-material
# Set the path to include the binaries and not just the default /usr/local/bin
ENV PATH="$PATH:/app/zk-material/bin"

# Copy the binaries from the base stage
COPY --from=base /app/zk-material/protocol/bin/ /app/zk-material/bin/


