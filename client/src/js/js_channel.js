function js_to_rs(message) {
    fetch('js_to_rs.wasm')
    .then(response => response.arrayBuffer())
    .then(bytes => WebAssembly.instantiate(bytes, {}))
    .then(results => {
        console.log(results);
    });
}

export function rs_to_js(message) {
    console.log(message);
}