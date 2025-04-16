FROM rust:latest as builder

WORKDIR /app
COPY . .

# Build the application
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/boopato /app/boopato

# Create volume mount point for the database
VOLUME /app/data

# Set environment variables
ENV DATABASE_PATH=/app/data/boopato.db

# Run the bot
CMD ["sh", "-c", "if [ ! -f \"$DATABASE_PATH\" ]; then touch \"$DATABASE_PATH\"; fi && ./boopato"]