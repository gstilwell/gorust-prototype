// see WASM websocket example at https://rustwasm.github.io/wasm-bindgen/examples/websockets.html

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use serde_json::json;
use serde::{Serialize, Deserialize};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize)]
struct IncomingMessage {
    MessageType: String,
    ClientId: u32,
}

#[derive(Serialize, Deserialize)]
struct WelcomeMessage {
    ClientId: u32,
}

pub struct Websocket {
    pub ws: WebSocket,
    pub client_id: u32,
}

impl Websocket {

pub fn new() -> Websocket {
    Websocket {
        ws: Websocket::start(),
        client_id: 10,
    }
}

pub fn send_message<T>(&self, payload: T)
where
    T: Serialize
{   
    let payload = json!(payload);

    match self.ws.send_with_str(&payload.to_string()) {
        Ok(_) => {}
        //TODO do something with error
        Err(err) => {}
    }
}

pub fn start() -> WebSocket {
    // Connect to an echo server
    let ws = Some(WebSocket::new("ws://localhost:5000/websocket").unwrap());
    // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
    ws.as_ref().unwrap().set_binary_type(web_sys::BinaryType::Arraybuffer);
    let cloned_ws = ws.as_ref().unwrap().clone();
    // create callback
    let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            console_log!("message event, received arraybuffer: {:?}", abuf);
            let array = js_sys::Uint8Array::new(&abuf);
            let len = array.byte_length() as usize;
            console_log!("Arraybuffer received {}bytes: {:?}", len, array.to_vec());


        } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
            console_log!("message event, received blob: {:?}", blob);
            // better alternative to juggling with FileReader is to use https://crates.io/crates/gloo-file
            let fr = web_sys::FileReader::new().unwrap();
            let fr_c = fr.clone();
            // create onLoadEnd callback
            let onloadend_cb = Closure::wrap(Box::new(move |_e: web_sys::ProgressEvent| {
                let array = js_sys::Uint8Array::new(&fr_c.result().unwrap());
                let len = array.byte_length() as usize;
                console_log!("Blob received {}bytes: {:?}", len, array.to_vec());
                // here you can for example use the received image/png data
            })
                as Box<dyn FnMut(web_sys::ProgressEvent)>);
            fr.set_onloadend(Some(onloadend_cb.as_ref().unchecked_ref()));
            fr.read_as_array_buffer(&blob).expect("blob not readable");
            onloadend_cb.forget();
        } else if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
            console_log!("message event, received Text: {:?}", txt);
            
            let txt_as_string: String = txt.into();
            let payload: IncomingMessage = serde_json::from_str(&txt_as_string).unwrap();

            if payload.MessageType == "welcome" {
                //let payload: WelcomeMessage = serde_json::from_str(&txt_as_string).unwrap();
                console_log!("welcome received! we are id {}", payload.ClientId);

                let ack = json!({
                    "messageType": "ack",
                    "clientId": 12,
                });
                match cloned_ws.send_with_str(&ack.to_string()) {
                    Ok(_) => {},
                    //TODO do something with error
                    Err(err) => {}
                }
            }
        } else {
            console_log!("message event, received Unknown: {:?}", e.data());
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    // set message event handler on WebSocket
    ws.as_ref().unwrap().set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
        console_log!("error event: {:?}", e);
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.as_ref().unwrap().set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let another_cloned_ws = ws.as_ref().unwrap().clone();
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        console_log!("socket opened");

        let hello = json!({
            "messageType": "salutations",
        });
        match another_cloned_ws.send_with_str(&hello.to_string()) {
            Ok(_) => {},
            //TODO do something with error
            Err(err) => {}
        }
    }) as Box<dyn FnMut(JsValue)>);
    ws.as_ref().unwrap().set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    ws.unwrap()
}
}