FROM gstilwell105/rust-wasm:rust-1.51
EXPOSE 8080
COPY . /app
WORKDIR /app
RUN wasm-pack build --target web
CMD ["basic-http-server", "-a", "0.0.0.0:8080"]