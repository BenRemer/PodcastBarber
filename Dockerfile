# 1. Switch to Ubuntu 24.04 (Noble) base image
FROM ubuntu:24.04 as builder

LABEL authors="ben"
WORKDIR /usr/src/app

# 2. Install Rust AND the required C/C++ build tools
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 3. Install the Rust toolchain manually since we aren't using the official rust image
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

COPY barber-api/ ./

# 4. Compile the application
RUN cargo build --release

# 5. The runner stage must ALSO be Ubuntu 24.04 to provide the matching glibc at runtime
FROM ubuntu:24.04

# 6. Install runtime dependencies
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

COPY --from=builder /usr/src/app/target/release/barber_api .

EXPOSE 3000

CMD ["./barber_api"]
