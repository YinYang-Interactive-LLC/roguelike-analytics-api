# First stage: Build
# Use an official Rust image for the build stage
FROM rust:1.82-bullseye AS builder

# Create a new empty shell project
WORKDIR /usr/src/roguelike-analytics-api
RUN USER=root cargo new --bin roguelike-analytics-api
WORKDIR /usr/src/roguelike-analytics-api

# Copy the Cargo manifest files
COPY ./Cargo.toml ./Cargo.lock ./

# Cache dependencies
RUN cargo fetch

# Copy the source code
COPY ./src ./src

# Create release build
RUN cargo build --release


# Second stage: Run
# Use a minimal Debian image
FROM debian:bullseye-slim

# Set environment variables
ENV APP_USER=appuser
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Install necessary packages
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

# Create a new user for running the application
RUN groupadd $APP_USER \
    && useradd -g $APP_USER -m -s /bin/bash $APP_USER

# Copy the binary from the build stage to the current stage
COPY --from=builder /usr/src/roguelike-analytics-api/target/release/roguelike-analytics-api /usr/local/bin/roguelike-analytics-api

# Copy docker-entrypoint
COPY deploy/docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod u+x /docker-entrypoint.sh

# Use an unprivileged user to run the app
USER $APP_USER

# Set the working directory
WORKDIR /home/$APP_USER

# Expose port
EXPOSE 8080

# Command to run the application
CMD ["/docker-entrypoint.sh"]
