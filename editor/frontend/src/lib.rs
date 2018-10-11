extern crate cfg_if;
extern crate wasm_bindgen;
extern crate web_sys;
extern crate js_sys;

mod utils;

use wasm_bindgen::prelude::*;
use web_sys::*;
use js_sys::Array;

use std::rc::Rc;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(a: &str);
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub fn greet() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("frame").unwrap();

    console_log!("{:?}", canvas);

    window.resize_to(500, 500).expect("Could not resize the window");
    window.alert_with_message("Suka blyat").unwrap();
}

#[wasm_bindgen]
pub struct Editor {
    context: Rc<WebGl2RenderingContext>,
}

#[wasm_bindgen]
impl Editor {
    pub fn update(&mut self) {

    }
}


#[wasm_bindgen]
pub fn create_editor(canvas: HtmlCanvasElement) -> Editor {
    let context: WebGl2RenderingContext = canvas.get_context("webgl2");
    
    let editor = Editor { context: Rc::new(context) };
    return editor;
}