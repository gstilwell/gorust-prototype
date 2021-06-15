use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use web_sys::console;
use js_sys::Uint8Array;

#[wasm_bindgen(module = "/src/js/js_channel.js")]
extern { fn rs_to_js(message: String); }

#[wasm_bindgen]
pub fn send(message: String)
{
    rs_to_js(message);
}

#[no_mangle]
pub extern "C" fn js_to_rs(message: String) {
    //recv_handler(message);
}