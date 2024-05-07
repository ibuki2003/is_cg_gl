use glm::Vec2;
use nalgebra_glm as glm;

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGl2RenderingContext as GL, *};

struct Scene {
    gl: GL,
    program: WebGlProgram,

    vao: WebGlVertexArrayObject,
    vbo_vtx: WebGlBuffer,
    vbo_col: WebGlBuffer,
    ibo_lin: WebGlBuffer,
    ibo_tri: WebGlBuffer,

    mvp_location: WebGlUniformLocation,

    index_count: i32,

    points: Vec<Vec2>,
}

const MAX_POINTS: usize = 1024;
impl Scene {
    fn new(canvas: &HtmlCanvasElement) -> Result<Self, JsValue> {
        canvas.set_width(CANVAS_SIZE);
        canvas.set_height(CANVAS_SIZE);
        let gl = canvas
            .get_context("webgl2")?
            .ok_or("Failed to get WebGl2RenderingContext")?
            .dyn_into::<GL>()?;

        let program = common::create_program(
            &gl,
            include_str!("shader/vertex.glsl"),
            include_str!("shader/fragment.glsl"),
        )?;

        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);
        gl.enable(GL::CULL_FACE);

        let vao = gl
            .create_vertex_array()
            .ok_or("Failed to create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        for i in 0..2 {
            gl.enable_vertex_attrib_array(i);
        }

        let vbo_vtx = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo_vtx));
        gl.buffer_data_with_i32(
            GL::ARRAY_BUFFER,
            4 * MAX_POINTS as i32 * 3,
            GL::DYNAMIC_DRAW,
        );
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);

        let vbo_col = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo_col));
        gl.buffer_data_with_i32(
            GL::ARRAY_BUFFER,
            4 * MAX_POINTS as i32 * 4,
            GL::DYNAMIC_DRAW,
        );
        gl.vertex_attrib_pointer_with_i32(1, 4, GL::FLOAT, false, 0, 0);

        let ibo_lin = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ibo_lin));
        gl.buffer_data_with_i32(
            GL::ELEMENT_ARRAY_BUFFER,
            2 * MAX_POINTS as i32 * 2,
            GL::DYNAMIC_DRAW,
        );

        let ibo_tri = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ibo_lin));
        gl.buffer_data_with_i32(
            GL::ELEMENT_ARRAY_BUFFER,
            2 * MAX_POINTS as i32 * 2,
            GL::DYNAMIC_DRAW,
        );

        let mvp_location = gl
            .get_uniform_location(&program, "mvpMatrix")
            .ok_or("Failed to get uniform location")?;

        let mut r = Self {
            gl,
            program,
            vao,
            vbo_vtx,
            vbo_col,
            ibo_lin,
            ibo_tri,

            mvp_location,
            index_count: 0,

            points: vec![
                Vec2::new(-0.5, 0.0),
                Vec2::new(0.0, 0.5),
                Vec2::new(0.5, 0.0),
            ],
        };

        r.update_spline();

        Ok(r)
    }

    fn update_spline(&mut self) {
        let spline = make_spline(&self.points, 0., 32);
        let n = self.points.len();

        let mut v = self
            .points
            .iter()
            .flat_map(|v| [v.x, v.y, 0.0])
            .collect::<Vec<_>>();
        v.extend(spline.iter().flat_map(|v| [v.x, v.y, 0.0]));
        send_vbo_data(&self.gl, &self.vbo_vtx, &v);

        let mut c = self
            .points
            .iter()
            .flat_map(|_| [1.0, 1.0, 1.0, 1.0])
            .collect::<Vec<_>>();
        c.extend((0..spline.len()).flat_map(|i| match i % 2 {
            0 => [1.0, 0.0, 1.0, 1.0],
            1 => [0.0, 1.0, 1.0, 1.0],
            _ => unreachable!(),
        }));
        send_vbo_data(&self.gl, &self.vbo_col, &c);

        let mut idx = (0..(self.points.len() - 1) as u16)
            .flat_map(|i| [i, i + 1])
            .collect::<Vec<_>>();
        idx.extend((0..(spline.len() - 1) as u16).flat_map(|i| [n as u16 + i, n as u16 + i + 1]));
        send_ibo_data(&self.gl, &self.ibo_lin, &idx);
        self.index_count = idx.len() as i32;
    }

    fn draw(&self) {
        self.gl.use_program(Some(&self.program));
        send_mvp_matrix(&self.gl, &self.mvp_location);

        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        self.gl
            .draw_elements_with_i32(GL::LINES, self.index_count, GL::UNSIGNED_SHORT, 0);
        self.gl.flush();
    }

    fn mousemove(&mut self, event: web_sys::MouseEvent) {
        if event.buttons() != 1 {
            return;
        }

        let p = Vec2::new(
            (event.offset_x() as f32 / CANVAS_SIZE as f32) * 2. - 1.,
            -(event.offset_y() as f32 / CANVAS_SIZE as f32) * 2. + 1.,
        );

        let i = self
            .points
            .iter()
            .enumerate()
            .min_by_key(|(_, x)| ((*x - p).norm_squared() * 1000.0) as i32)
            .map(|(i, _)| i)
            .unwrap();

        self.points[i] = p;

        self.update_spline();
    }
}

const CANVAS_SIZE: u32 = 1024;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document
        .get_element_by_id("canvas")
        .ok_or("canvas not found")?
        .dyn_into::<HtmlCanvasElement>()?;

    let scene = Rc::new(RefCell::new(Scene::new(&canvas)?));

    let scene_ = scene.clone();
    let handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        scene_.borrow_mut().mousemove(event);
    }) as Box<dyn FnMut(_)>);

    canvas.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())?;
    handler.forget();

    let closure = Rc::new(RefCell::new(None));
    let closure_ = closure.clone();
    *closure_.borrow_mut() = Some(Closure::<dyn FnMut() -> Result<i32, JsValue>>::new(
        move || {
            scene.borrow().draw();
            request_animation_frame(closure.borrow().as_ref().unwrap())
        },
    ));
    request_animation_frame(closure_.borrow().as_ref().unwrap())?;

    Ok(())
}

fn send_vbo_data(gl: &GL, vbo: &WebGlBuffer, data: &[f32]) {
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(vbo));
    let view = unsafe { js_sys::Float32Array::view(data) };
    gl.buffer_sub_data_with_i32_and_array_buffer_view(GL::ARRAY_BUFFER, 0, &view);
}

fn send_ibo_data(gl: &GL, ibo: &WebGlBuffer, data: &[u16]) {
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(ibo));
    let view = unsafe { js_sys::Uint16Array::view(data) };
    gl.buffer_sub_data_with_i32_and_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, 0, &view);
}

fn make_spline(points: &[Vec2], _p: f32, n: usize) -> Vec<Vec2> {
    let mut r = vec![];
    r.extend((0..n).map(|j| {
        let t = j as f32 / (n - 1) as f32;
        let p0 = points[0] * (1. - t) + points[1] * t;
        let p1 = points[1] * (1. - t) + points[2] * t;
        p0 * (1. - t) + p1 * t
    }));
    r
}

fn send_mvp_matrix(gl: &GL, location: &WebGlUniformLocation) {
    let mvp_matrix = glm::Mat4::identity();
    let mvp_arrays: [[f32; 4]; 4] = mvp_matrix.into();
    let mvp_matrices = mvp_arrays.iter().flat_map(|a| *a).collect::<Vec<_>>();

    gl.uniform_matrix4fv_with_f32_array_and_src_offset_and_src_length(
        Some(location),
        false,
        &mvp_matrices,
        0,
        0,
    );
}

fn request_animation_frame(
    closure: &Closure<dyn FnMut() -> Result<i32, JsValue>>,
) -> Result<i32, JsValue> {
    let window = web_sys::window().unwrap();
    window.request_animation_frame(closure.as_ref().unchecked_ref())
}
