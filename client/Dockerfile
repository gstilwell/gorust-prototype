FROM rust:1.51
WORKDIR /usr/src/client
COPY . .
RUN cargo install wasm-pack
RUN cargo install basic-http-server
RUN wasm-pack build --target web
CMD ["basic-http-server"]