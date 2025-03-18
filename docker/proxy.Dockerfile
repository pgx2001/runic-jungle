FROM rust:latest AS builder

# Install dependencies
RUN apt-get update && apt-get install -y git && rm -rf /var/lib/apt/lists/*

# Clone the repository
WORKDIR /app
RUN git clone https://github.com/octopus-network/idempotent-proxy.git
WORKDIR /app/idempotent-proxy
RUN git checkout runes-indexer

# Build the idempotent-proxy-server
RUN cargo install --path src/idempotent-proxy-server

# Create a new minimal image
FROM debian:latest

# Install required libraries
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /usr/local/cargo/bin/idempotent-proxy-server /usr/local/bin/idempotent-proxy-server

# Set environment variables (override with docker run -e if needed)
ENV USER=icp:test
ENV URL_LOCAL=http://127.0.0.1:18443

# Expose the port (change if needed)
EXPOSE 8080

# Command to run the server
CMD ["idempotent-proxy-server", "--port", "8080"]
