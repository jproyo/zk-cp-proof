# Multistage build to reduce image size
# First stage builds the binary
FROM rust:1.78-slim-bookworm as base

RUN apt update && \
    apt install -y make protobuf-compiler iproute2 iputils-ping iperf net-tools dnsutils ssh git gcc libssl-dev libprotobuf-dev

WORKDIR /app/zk-verifier
COPY . .

# Install the binary leaving it in the WORKDIR/bin folder
RUN mkdir -p /app/zk-verifier/bin
RUN cargo install --path verifier --root verifier --bins

# Second stage builds the runtime image.
# This stage will be the final image
FROM debian:stable-slim as go-runtime

RUN apt update && \
    apt install -y iproute2 iputils-ping iperf net-tools dnsutils libssl-dev libprotobuf-dev curl netcat-openbsd
WORKDIR /app/zk-verifier
RUN mkdir -p /app/zk-verifier/config

# Set the path to include the binaries and not just the default /usr/local/bin
ENV PATH="$PATH:/app/zk-verifier/bin"

# We are going to need grpc-health-probe to check the health of the grpc server for docker-compose or future deployments
# Install go because grpc-health-probe is written in go and we need to compile it
RUN curl -OL https://go.dev/dl/go1.21.6.linux-amd64.tar.gz
RUN rm -rf /usr/local/go && tar -C /usr/local -xzf go1.21.6.linux-amd64.tar.gz
RUN rm go1.21.6.linux-amd64.tar.gz
ENV PATH="$PATH:/usr/local/go/bin:/root/go/bin"
# Install grpc-health-probe
RUN go install github.com/grpc-ecosystem/grpc-health-probe@latest

# Third stage: Copy the binaries from the base stage and the go-runtime stage
FROM debian:stable-slim as runtime
WORKDIR /app/zk-verifier
# Set the path to include the binaries and not just the default /usr/local/bin
ENV PATH="$PATH:/app/zk-verifier/bin"

# Copy the binaries from the base stage
COPY --from=base /app/zk-verifier/verifier/bin/ /app/zk-verifier/bin/
COPY --from=go-runtime /root/go/bin/grpc-health-probe /app/zk-verifier/bin/grpc-health-probe
COPY ./verifier/config/default.toml /app/zk-verifier/config/default.toml


