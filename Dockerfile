# Stage 1: Build Rust WASM output
FROM rust:1.87 AS builder
WORKDIR /app

# Install trunk and wasm-bindgen-cli
RUN cargo install trunk wasm-bindgen-cli

# Copy your entire project source code
COPY . .

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Build your site for release (Trunk)
RUN trunk build --release

# Stage 2: Minimal Nginx Web Server
FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY www/nginx.conf /etc/nginx/nginx.conf
COPY www/ssl /etc/nginx/ssl
EXPOSE 80
EXPOSE 443
CMD ["nginx", "-g", "daemon off;"]