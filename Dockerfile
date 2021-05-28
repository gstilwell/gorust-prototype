# generates a docker container that can be used for building and hosting webassembly applications

FROM rust:1.51
RUN cargo install wasm-pack
RUN cargo install basic-http-server