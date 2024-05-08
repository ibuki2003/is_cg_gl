use common::vao::MyVAO;
use glm::Vec2;
use nalgebra_glm as glm;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGl2RenderingContext as GL, *};

mod curves;

struct Scene {
    gl: Rc<GL>,
    program: WebGlProgram,

    vao_lin: MyVAO,
    vao_tri: MyVAO,

    mvp_location: WebGlUniformLocation,

    points: Vec<Vec2>,

    splitnum: usize,
    curvetype: curves::CurveType,

    dragging: Option<usize>,
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

            points: vec![
                Vec2::new(-0.4, -0.5),
                Vec2::new(0.5, 0.5),
                Vec2::new(-0.5, 0.5),
                Vec2::new(0.4, -0.5),
            ],

            splitnum: 16,
            curvetype: curves::CurveType::Bezier,

            dragging: None,
        };

        r.update();

        Ok(r)
    }

    fn update(&mut self) {
        let spline = curves::make_curve(&self.points, self.splitnum, self.curvetype);
        let n = self.points.len();

        let mut v = self
            .points
            .iter()
            .flat_map(|v| [v.x, v.y, 0.0])
            .collect::<Vec<_>>();
        v.extend(spline.iter().flat_map(|v| [v.x, v.y, 0.0]));

        let mut c = self
            .points
            .iter()
            .flat_map(|_| [1.0, 1.0, 1.0, 1.0])
            .collect::<Vec<_>>();
        c.extend((0..spline.len()).flat_map(|i| {
            let t = i as f32 / (spline.len() - 1) as f32;
            [1.0, 1.0 - t, t, 1.0]
        }));

        let mut idx = (0..(self.points.len() - 1) as u16)
            .flat_map(|i| [i, i + 1])
            .collect::<Vec<_>>();
        idx.extend((0..(spline.len() - 1) as u16).flat_map(|i| [n as u16 + i, n as u16 + i + 1]));
        self.vao_lin.send_data(&v, &c, &idx);

        const DOT_SIZE: f32 = 0.005;
        let mut v = spline
            .iter()
            .flat_map(|v| {
                [
                    v.x - DOT_SIZE,
                    v.y - DOT_SIZE,
                    0.0,
                    v.x + DOT_SIZE,
                    v.y - DOT_SIZE,
                    0.0,
                    v.x,
                    v.y + DOT_SIZE,
                    0.0,
                ]
            })
            .collect::<Vec<_>>();
        let mut c = [1.0, 0.0, 0.0, 1.0].repeat(spline.len() * 3);

        v.extend(self.points.iter().flat_map(|v| {
            [
                v.x - DOT_SIZE,
                v.y - DOT_SIZE,
                0.0,
                v.x + DOT_SIZE,
                v.y - DOT_SIZE,
                0.0,
                v.x,
                v.y + DOT_SIZE,
                0.0,
            ]
        }));
        c.extend([0.0, 1.0, 0.0, 1.0].repeat(self.points.len() * 3));

        let idx = (0..((spline.len() + self.points.len()) * 3) as u16).collect::<Vec<_>>();

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
            self.dragging = None;
            return;
        }

        let p = Vec2::new(
            (event.offset_x() as f32 / CANVAS_SIZE as f32) * 2. - 1.,
            -(event.offset_y() as f32 / CANVAS_SIZE as f32) * 2. + 1.,
        );

        let i = match self.dragging {
            None => {
                let i = self
                    .points
                    .iter()
                    .map(|v| (v - p).norm_squared())
                    .enumerate()
                    .min_by(|(_, a), (_, b)| {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)
                    .unwrap();
                self.dragging = Some(i);
                i
            },
            Some(i) => i,
        };

        self.points[i] = p;

        self.update();
    }

    fn set_splitnum(&mut self, n: usize) {
        if !(2..MAX_POINTS).contains(&n) {
            return;
        }

        self.splitnum = n;
        self.update();
    }

    fn set_order(&mut self, d: usize) {
        if !(2..=MAX_POINTS).contains(&d) {
            return;
        }

        let n = d + 1;
        let m = self.points.len();
        if n == m {
            return;
        }

        let last = *self.points.last().unwrap();

        if m < n {
            let a = self.points[m - 2];
            self.points.truncate(m - 1); // drop last
            self.points.extend((1..n - m + 2).map(|i| {
                let t = i as f32 / (n - m + 1) as f32;
                a * (1. - t) + last * t
            }));
        } else {
            self.points.truncate(n);
            self.points[n - 1] = last;
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
    canvas.add_event_listener_with_callback("mouseup", handler.as_ref().unchecked_ref())?;
    canvas.add_event_listener_with_callback("mousedown", handler.as_ref().unchecked_ref())?;
    handler.forget();

    // input handlers
    let scene_ = scene.clone();
    let handler = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let targ = if let Some(e) = event.target() {
            e
        } else {
            return;
        };
        let targ = targ.dyn_into::<HtmlInputElement>();
        let targ = if let Ok(e) = targ { e } else { return };

        let targid = targ.name();
        let val = targ.value();

        match &*targid {
            "splitnum" => {
                scene_.borrow_mut().set_splitnum(val.parse().unwrap());
            }
            "order" => {
                scene_.borrow_mut().set_order(val.parse().unwrap());
            }
            "curvetype" => {
                let mut scene = scene_.borrow_mut();
                scene.curvetype = match val.as_str() {
                    "bezier" =>  curves::CurveType::Bezier,
                    "catmullrom_uniform" => curves::CurveType::CatmullRom(curves::CatmullRomParmType::Uniform),
                    "catmullrom_chordal" => curves::CurveType::CatmullRom(curves::CatmullRomParmType::ChordLength),
                    "catmullrom_centripetal" => curves::CurveType::CatmullRom(curves::CatmullRomParmType::Centripetal),
                    _ => { return; }
                };
                scene.update();

            }
            _ => {}
        }
    }) as Box<dyn FnMut(_)>);
    document.add_event_listener_with_callback("change", handler.as_ref().unchecked_ref())?;

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
