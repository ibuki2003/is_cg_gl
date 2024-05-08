extern crate nalgebra_glm as glm;

use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGl2RenderingContext as GL, *};

pub mod vao;

pub fn create_program(
    gl: &GL,
    source_vertex: &'static str,
    source_fragment: &'static str,
) -> Result<WebGlProgram, String> {
    let vertex_shader = create_shader(gl, GL::VERTEX_SHADER, source_vertex)?;
    let fragment_shader = create_shader(gl, GL::FRAGMENT_SHADER, source_fragment)?;

    let program = gl
        .create_program()
        .ok_or("Failed to create program object")?;
    gl.attach_shader(&program, &vertex_shader);
    gl.attach_shader(&program, &fragment_shader);
    gl.link_program(&program);

    if gl
        .get_program_parameter(&program, GL::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        let log = gl
            .get_program_info_log(&program)
            .unwrap_or(String::from("Failed to link program"));
        gl.delete_program(Some(&program));
        Err(log)
    }
}

pub fn create_shader(gl: &GL, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or("Failed to create shader object")?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        let log = gl
            .get_shader_info_log(&shader)
            .unwrap_or(String::from("Failed to compile shader"));
        gl.delete_shader(Some(&shader));
        Err(log)
    }
}

pub fn request_animation_frame(
    closure: &Closure<dyn FnMut() -> Result<i32, JsValue>>,
) -> Result<i32, JsValue> {
    let window = web_sys::window().unwrap();
    window.request_animation_frame(closure.as_ref().unchecked_ref())
}
