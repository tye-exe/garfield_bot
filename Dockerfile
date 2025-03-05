# Use the official Rust image as a base
FROM rust

# Set the working directory to /app
WORKDIR /app

# Copy the Cargo.toml file into the working directory
COPY . .

# Build the Rust application
RUN cargo build --release

# Copy the built application into the working directory
# COPY target/release/* .

# Expose the port that the application will listen on
# EXPOSE 8080

# Run the command to start the application when the container is launched
CMD ["./target/release/garfield_bot"]
