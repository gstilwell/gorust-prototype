# creates container that builds and hosts the gorust game client.
# dependencies and tools are already loaded in upstream containers

FROM gstilwell105/gorust-client-dependencies:latest
EXPOSE 8080
WORKDIR /app
COPY . .
RUN wasm-pack build --target web
CMD ["basic-http-server", "-a", "0.0.0.0:8080"]