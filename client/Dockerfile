FROM rust:1.51
EXPOSE 8080
WORKDIR /usr/src/client
COPY . .
WORKDIR /usr/src/client/client
RUN cargo install wasm-pack
RUN cargo install basic-http-server
RUN wasm-pack build --target web
CMD ["basic-http-server", "-a", "127.0.0.1:8080"]