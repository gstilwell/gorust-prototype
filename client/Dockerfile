#FROM gstilwell105/rust-wasm:rust-1.51
FROM gstilwell105/gorust-client-dependencies:latest
EXPOSE 8080
WORKDIR /app
COPY . .
RUN wasm-pack build --target web
CMD ["basic-http-server", "-a", "0.0.0.0:8080"]