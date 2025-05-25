# Stage 1: Build Rust WASM output
FROM rust:1.87 as builder
WORKDIR /app

# Install trunk and wasm-bindgen
RUN cargo install trunk wasm-bindgen-cli

# Copy source code
COPY . .

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Build the site with trunk
RUN trunk build --release

# Stage 2: Serve with nginx
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY www/nginx.conf /etc/nginx/nginx.conf  # We'll create this file next
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
