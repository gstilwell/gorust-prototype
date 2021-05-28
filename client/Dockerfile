FROM rust:1.51
EXPOSE 4000
WORKDIR /usr/src/client
COPY . .
WORKDIR /usr/src/client/client
RUN cargo install wasm-pack
RUN cargo install basic-http-server
RUN wasm-pack build --target web
CMD ["basic-http-server"]