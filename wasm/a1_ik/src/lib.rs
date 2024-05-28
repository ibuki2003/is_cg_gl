use common::vao::MyVAO;
use glm::{rotate_vec2, Vec2};
use nalgebra_glm as glm;

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGl2RenderingContext as GL, *};

mod ik;

struct Scene {
    gl: Rc<GL>,
    program: WebGlProgram,

    vao_lin: MyVAO,
    vao_tri: MyVAO,

    mvp_location: WebGlUniformLocation,

    ik: ik::IK,
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
        let gl = Rc::new(gl);

        let program = common::create_program(
            &gl,
            include_str!("shader/vertex.glsl"),
            include_str!("shader/fragment.glsl"),
        )?;

        gl.enable(GL::DEPTH_TEST);
        gl.depth_func(GL::LEQUAL);
        gl.enable(GL::CULL_FACE);

        let vao_lin = MyVAO::new(gl.clone(), MAX_POINTS, MAX_POINTS * 2)?;
        let vao_tri = MyVAO::new(gl.clone(), MAX_POINTS, MAX_POINTS * 3)?;

        let mvp_location = gl
            .get_uniform_location(&program, "mvpMatrix")
            .ok_or("Failed to get uniform location")?;

        let mut r = Self {
            gl,
            program,
            vao_lin,
            vao_tri,

            mvp_location,

            ik: ik::IK::new(),
        };

        r.update();

        Ok(r)
    }

    fn update(&mut self) {
        let p = self.ik.render();
        let n = p.len();

        let mut v = vec![];

        v.extend(p.windows(2).flat_map(|v| {
            let v0 = v[0];
            let v3 = v[1];
            let d = v3 - v0;
            let v1 = v0 + rotate_vec2(&d, 0.5) * 0.2;
            let v2 = v0 + rotate_vec2(&d, -0.5) * 0.2;
            [
                v0.x, v0.y, 0.0, v1.x, v1.y, 0.0, v2.x, v2.y, 0.0, v3.x, v3.y, 0.0,
            ]
        }));

        let mut c = p
            .iter()
            .flat_map(|_| [1.0, 1.0, 1.0, 1.0])
            .collect::<Vec<_>>();

        let mut idx = (0..(p.len() - 1) as u16)
            .flat_map(|i| [i * 4, i * 4 + 3, i * 4 + 1, i * 4, i * 4 + 2, i * 4 + 3])
            .collect::<Vec<_>>();
        self.vao_tri.send_data(&v, &c, &idx);
    }

    fn draw(&self) {
        self.gl.use_program(Some(&self.program));
        send_mvp_matrix(&self.gl, &self.mvp_location);

        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        self.vao_lin.draw_elements(GL::LINES);
        self.vao_tri.draw_elements(GL::TRIANGLES);
        self.gl.flush();
    }

    fn mouse_handler(&mut self, event: web_sys::MouseEvent) {
        if event.buttons() != 1 {
            return;
        }

        let p = Vec2::new(
            (event.offset_x() as f32 / CANVAS_SIZE as f32) * 2. - 1.,
            -(event.offset_y() as f32 / CANVAS_SIZE as f32) * 2. + 1.,
        );

        // todo!();

        self.ik.update(p);

        self.update();
    }

    fn scroll_handler(&mut self, event: web_sys::WheelEvent) {
        let delta: f64 = event.delta_y();
        if delta.abs() < 1. {
            return;
        }
        // todo!();
        let p = Vec2::new(
            (event.offset_x() as f32 / CANVAS_SIZE as f32) * 2. - 1.,
            -(event.offset_y() as f32 / CANVAS_SIZE as f32) * 2. + 1.,
        );

        // console::log_1(&format!("delta: {}", delta).into());

        let ps = self.ik.render();
        let e = ps
            .windows(2)
            .map(|v| ((v[1] + v[0]) * 0.5 - p).magnitude_squared());
        let nearest = e
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .unwrap()
            .0;

        let l = &mut self.ik.arms[nearest].length;
        *l = (*l - 0.1 * delta.signum() as f32).max(0.1);

        self.update();
    }

    fn addrmv(&mut self, d: i32) {
        if d < 0 {
            for _ in 0..d.abs() {
                self.ik.pop_arm();
            }
        } else {
            for _ in 0..d {
                self.ik.add_arm();
            }
        }
        self.update();
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

    // mousemove handler
    let scene_ = scene.clone();
    let handler = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        scene_.borrow_mut().mouse_handler(event);
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("mousemove", handler.as_ref().unchecked_ref())?;
    canvas.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())?;
    handler.forget();

    let scene_ = scene.clone();
    let handler = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
        scene_.borrow_mut().scroll_handler(event);
    }) as Box<dyn FnMut(_)>);
    canvas.add_event_listener_with_callback("wheel", handler.as_ref().unchecked_ref())?;
    handler.forget();

    let btn_add = document
        .get_element_by_id("btn_add")
        .ok_or("btn_add not found")?
        .dyn_into::<HtmlButtonElement>()?;
    let scene_ = scene.clone();
    let handler = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
        scene_.borrow_mut().addrmv(1);
    }) as Box<dyn FnMut(_)>);
    btn_add.add_event_listener_with_callback("click", handler.as_ref().unchecked_ref())?;
    handler.forget();

    let btn_rmv = document
        .get_element_by_id("btn_rmv")
        .ok_or("btn_rmv not found")?
        .dyn_into::<HtmlButtonElement>()?;
    let scene_ = scene.clone();
    let handler = Closure::wrap(Box::new(move |event: web_sys::WheelEvent| {
        scene_.borrow_mut().addrmv(-1);
    }) as Box<dyn FnMut(_)>);
    btn_rmv.add_event_listener_with_callback("click", handler.as_ref().unchecked_ref())?;
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
