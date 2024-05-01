use nalgebra_glm as glm;

use std::{cell::RefCell, f32::consts, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGl2RenderingContext as GL, *};

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("canvas not found")?
        .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(768);
    canvas.set_height(768);

    let gl = canvas
        .get_context("webgl2")?
        .ok_or("Failed to get WebGl2RenderingContext")?
        .dyn_into::<GL>()?;

    todo!()
}

