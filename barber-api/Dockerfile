FROM rust:bookworm as builder

LABEL authors="ben"

WORKDIR /usr/src/app
COPY . .

# Compile the application for release
RUN cargo build --release

FROM debian:bookworm-slim

# Install OpenSSL certificates (required for reqwest to make HTTP calls)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin
# Copy ONLY the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/barber_api .

# Expose the Axum port
EXPOSE 3000

# Run the binary
CMD ["./barber_api"]