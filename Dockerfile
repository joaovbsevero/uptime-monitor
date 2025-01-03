FROM rust:latest as builder

WORKDIR /app

# Conditionally copy .env file if it exists
RUN if [ -f ./.env ]; then \
        cp ./.env /app/.env; \
    else \
        touch .env; \
    fi

# Copy resources, code and config files
COPY ./resources ./resources
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src

# Build your application
RUN cargo build --release

# Use a minimal base image for the final container
FROM debian:bookworm-slim

WORKDIR /app

# Install necessary libraries
RUN apt update && apt install -y libssl3

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/uptime-monitor .
COPY --from=builder /app/.env .env

# Run the binary
CMD ./uptime-monitor
