use serde::{Deserialize, Serialize};
use serde_json;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Serialize, Deserialize)]
pub struct BitsocketResp {
    #[serde(rename = "type")]
    t: String,
    data: String,
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    // Your code goes here!
    console::log_1(&JsValue::from_str("Hello world!"));

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    // Canvas
    let canvas = document.get_element_by_id("canvas").unwrap();

    let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    context.set_fill_style(&JsValue::from_str("gold"));

    context.fill_rect(0.0, 0.0, 100.0, 100.0);

    // EventSource
    let source = web_sys::EventSource::new("https://genesis.bitdb.network/s/1FnauZ9aUH2Bex6JzdcV4eNX7oLSSEbxtN/ewogICJ2IjogMywKICAicSI6IHsKICAgICJmaW5kIjoge30KICB9LAogICJyIjogewogICAgImYiOiAiLltdIHwgLnR4LmgiCiAgfQp9").unwrap();

    let callback = Closure::wrap(Box::new(move |x: web_sys::MessageEvent| {
        let s = x.data().dyn_into::<js_sys::JsString>().unwrap();
        // console::log_1(&s);

        let d: JsValue = x.data();
        let resp: BitsocketResp = serde_json::from_str(&d.as_string().unwrap()).unwrap();
        console::log_1(&JsValue::from_str(&resp.data));

        // change box color

        let string = &s.as_string().unwrap()[25..31];
        context.set_fill_style(&JsValue::from_str(&format!("#{}", string)));
        context.fill_rect(0.0, 0.0, 100.0, 100.0)
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);

    source.set_onmessage(Some(callback.as_ref().unchecked_ref()));

    callback.forget();

    Ok(())
}
