use serde::{Deserialize, Serialize};
use serde_json;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
// use secp256k1;

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

pub struct SmoothQueue {
    last_len: usize,
    interval: i32,
    queue: VecDeque<String>,
}

impl SmoothQueue {
    fn new() -> Self {
        SmoothQueue {
            last_len: 0,
            interval: 1000,
            queue: VecDeque::new(),
        }
    }

    fn push(&mut self, job: String) {
        self.queue.push_back(job);
    }

    fn pop(&mut self) -> Option<String> {
        let len = self.queue.len();
        if len > self.last_len {
            self.interval = self.interval / 10 * 11;
        } else if len < self.last_len {
            self.interval = self.interval / 10 * 9;
        }
        self.last_len = len;
        self.queue.pop_front()
    }

    // fn interval(&self) -> i32 {
    //     self.interval
    // }
}

pub fn insert_header(
    document: &web_sys::Document,
    body: &web_sys::HtmlElement,
    content: String,
) -> Result<(), JsValue> {
    let h1 = document.create_element("h1")?;
    h1.set_inner_html(&content);

    let h1 = h1.dyn_into::<web_sys::HtmlElement>().unwrap();
    h1.style().set_property("color", "purple")?;

    body.append_child(&h1)?;

    Ok(())
}

pub fn subscribe(
    id: String,
    document: &web_sys::Document,
    a: Option<&js_sys::Function>,
) -> web_sys::HtmlElement {
    let t = document.create_element("button").unwrap();
    let t = t.dyn_into::<web_sys::HtmlElement>().unwrap();
    t.set_id(&id);
    t.set_onclick(a);
    t.style().set_property("display", "none").unwrap();
    let body = document.body().unwrap();
    body.append_child(&t).unwrap();
    t
}

pub fn send(id: String, document: &web_sys::Document) {
    let t = document.get_element_by_id(&id).unwrap();
    let t = t.dyn_into::<web_sys::HtmlElement>().unwrap();
    t.click();
}

pub struct Counter {
    count: i32,
}

impl Counter {
    fn new() -> Self {
        Counter { count: 0 }
    }

    fn click(&mut self) {
        self.count += 1;
    }

    fn count_string(&self) -> String {
        self.count.to_string()
    }
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

    let window = web_sys::window()
        .unwrap()
        .dyn_into::<web_sys::Window>()
        .unwrap();
    let document = window.document().unwrap();

    // header
    let body = document.body().unwrap();

    insert_header(&document, &body, "Bitcoin Beat Box".to_string()).unwrap();

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

    // SmoothQueue
    let queue = Rc::new(RefCell::new(SmoothQueue::new()));
    let queue1 = queue.clone();

    let pop_job_callback = Closure::wrap(Box::new(move || match queue1.borrow_mut().pop() {
        Some(job) => {
            console::log_1(&JsValue::from_str(&job));
            context.set_fill_style(&JsValue::from_str(&format!("#{}", &job)));
            context.fill_rect(0.0, 0.0, 100.0, 100.0);
        }
        None => {
            console::log_1(&JsValue::from_str("empty"));
        }
    }) as Box<dyn FnMut()>);

    subscribe(
        "newtx".to_string(),
        &document,
        Some(pop_job_callback.as_ref().unchecked_ref()),
    );
    pop_job_callback.forget();

    // EventSource
    let source = web_sys::EventSource::new("https://genesis.bitdb.network/s/1FnauZ9aUH2Bex6JzdcV4eNX7oLSSEbxtN/ewogICJ2IjogMywKICAicSI6IHsKICAgICJmaW5kIjoge30KICB9LAogICJyIjogewogICAgImYiOiAiLltdIHwgLnR4LmgiCiAgfQp9").unwrap();

    let document1 = document.clone();
    // called when got new tx
    let callback = Closure::wrap(Box::new(move |x: web_sys::MessageEvent| {
        let s = x.data().dyn_into::<js_sys::JsString>().unwrap();
        // console::log_1(&s);

        let d: JsValue = x.data();
        let resp: BitsocketResp = serde_json::from_str(&d.as_string().unwrap()).unwrap();
        console::log_1(&JsValue::from_str(&resp.data));
        // enqueue
        let string = &s.as_string().unwrap()[25..31];

        queue.borrow_mut().push(string.to_string());
        send("newtx".to_string(), &document1);
    }) as Box<dyn FnMut(web_sys::MessageEvent)>);

    source.set_onmessage(Some(callback.as_ref().unchecked_ref()));

    callback.forget();

    // Counter
    let counter_text = document.create_element("p")?;
    let counter_btn = document.create_element("button")?;

    body.append_child(&counter_text)?;
    body.append_child(&counter_btn)?;

    let mut counter = Counter::new();
    let add_one = Closure::wrap(Box::new(move || {
        counter.click();
        counter_text.set_inner_html(&counter.count_string());
    }) as Box<dyn FnMut()>);

    let counter_btn = counter_btn.dyn_into::<web_sys::HtmlElement>()?;
    counter_btn.set_onclick(Some(add_one.as_ref().unchecked_ref()));
    counter_btn.set_inner_html("Click");

    add_one.forget();

    Ok(())
}
