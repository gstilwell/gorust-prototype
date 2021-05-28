# gorust-prototype
prototype for a rust client/golang server 3D video game with starry eyes for webassembly

to install tools:
cargo install wasm-pack
cargo install basic-http-server

to build client:
wasm-pack build --target web

to run client:
basic-http-server -a 127.0.0.1:8080
http://localhost:8080