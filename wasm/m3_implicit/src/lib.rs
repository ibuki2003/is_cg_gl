use cgmath::{Point3, Vector3};
use cgmath::prelude::{InnerSpace, EuclideanSpace};
use common::vao::MyVAO;
use nalgebra_glm as glm;
use std::{cell::RefCell, f32::consts, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{WebGl2RenderingContext as GL, *};


use half_edge_mesh::HalfEdgeMesh;

struct Scene {
    gl: Rc<GL>,
    program: WebGlProgram,

    vao_lin: MyVAO,
    vao_tri: MyVAO,

    mvp_location: WebGlUniformLocation,

    camera_dist: f32,
    camera_pitch: f32,
    camera_yaw: f32,

    mesh: HalfEdgeMesh,
}

fn make_tetrahedron() -> HalfEdgeMesh {
    HalfEdgeMesh::from_tetrahedron_pts(
        Point3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Point3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        Point3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        Point3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    )
}

// run Marching Tetrahedra algorithm
fn make_model_implicit(range: f32, split: usize, func: fn(&Point3<f32>) -> f32) -> HalfEdgeMesh {
    let grid_size = range * 2.0 / split as f32;

    let mut vertices = Vec::new();

    // find intersections
    const EDGE_DIRECTIONS: [(usize, usize, usize); 7] = [
        (1, 0, 0), // X
        (0, 1, 0), // Y
        (0, 0, 1), // Z
        (1, 1, 0), // XY
        (0, 1, 1), // YZ
        (1, 0, 1), // XZ
        (1, 1, 1), // XYZ
    ];
    let verts = (0..=split)
        .map(|x| {
            (0..=split)
                .map(|y| {
                    (0..=split)
                        .map(|z| {
                            EDGE_DIRECTIONS
                                .iter()
                                .map(|(dx, dy, dz)| {
                                    let p0 = Point3 {
                                        x: x as f32 * grid_size - range,
                                        y: y as f32 * grid_size - range,
                                        z: z as f32 * grid_size - range,
                                    };
                                    let p1 = Point3 {
                                        x: (x + dx) as f32 * grid_size - range,
                                        y: (y + dy) as f32 * grid_size - range,
                                        z: (z + dz) as f32 * grid_size - range,
                                    };
                                    let v0 = func(&p0);
                                    let v1 = func(&p1);
                                    if v0 * v1 < 0.0 {
                                        let t = v0 / (v0 - v1);
                                        let id = vertices.len();
                                        vertices.push(p0 + (p1 - p0) * t);
                                        Some(id)
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // add faces
    const GRID_FACES: [[usize; 3]; 18] = [
        [4, 6, 17],
        [1, 6, 12],
        [3, 6, 16],
        [0, 6, 9],
        [5, 6, 18],
        [2, 6, 15],
        [2, 4, 14],
        [1, 4, 11],
        [11, 12, 17],
        [10, 12, 16],
        [7, 9, 16],
        [8, 9, 18],
        [0, 5, 8],
        [2, 5, 13],
        [1, 3, 10],
        [0, 3, 7],
        [14, 15, 17],
        [13, 15, 18],
    ];

    const GRID_BODIES: [[usize; 4]; 6] = [
        [0, 5, 6, 16],
        [0, 1, 7, 8],
        [1, 2, 9, 14],
        [2, 3, 10, 15],
        [3, 4, 11, 12],
        [4, 5, 13, 17],
    ];

    let mut faces = Vec::new();

    for x in 0..split {
        for y in 0..split {
            for z in 0..split {
                let mut grid_edges = verts[x][y][z].clone();
                grid_edges.extend([
                    // X
                    verts[x + 1][y][z][1], // Y
                    verts[x + 1][y][z][2], // Z
                    verts[x + 1][y][z][4], // YZ
                    // Y
                    verts[x][y + 1][z][0], // X
                    verts[x][y + 1][z][2], // Z
                    verts[x][y + 1][z][5], // XZ
                    // Z
                    verts[x][y][z + 1][0],     // X
                    verts[x][y][z + 1][1],     // Y
                    verts[x][y][z + 1][3],     // XY
                    verts[x + 1][y + 1][z][2], // XY -> Z
                    verts[x][y + 1][z + 1][0], // YZ -> X
                    verts[x + 1][y][z + 1][1], // XZ -> Y
                ]);

                let edges = GRID_FACES
                    .iter()
                    .map(|i| {
                        let v = i.iter().filter_map(|&i| grid_edges[i]).collect::<Vec<_>>();
                        // assert_eq!(v.len() % 2, 0);
                        if v.len() == 2 {
                            Some((v[0], v[1]))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();

                for i in GRID_BODIES {
                    let mut f = i.iter().filter_map(|&i| edges[i]).collect::<Vec<_>>();
                    if f.is_empty() { continue; }
                    // assert!(f.len() != 1 && f.len() != 2);
                    if f.len() < 3 { continue; }
                    let e0 = f.pop().unwrap();
                    let (mut pr, mut cr) = e0;
                    let mut e = vec![pr, cr];
                    for _ in 0..f.len() {
                        if cr == e0.0 { break; }
                        for ff in f.iter() {
                            if ff.0 == cr && ff.1 != pr {
                                e.push(ff.1);
                                pr = cr;
                                cr = ff.1;
                                break;
                            }
                            if ff.1 == cr && ff.0 != pr {
                                e.push(ff.0);
                                pr = cr;
                                cr = ff.0;
                                break;
                            }
                        }
                    }
                    if e.first() == e.last() {
                        e.pop();
                    }

                    if e.len() == 4 {
                        e = vec![
                            e[0], e[1], e[3],
                            e[1], e[2], e[3],
                        ];
                    }
                    for es in e.chunks(3) {
                        faces.push([es[0], es[1], es[2]]);
                    }
                }
            }
        }
    }

    HalfEdgeMesh::from_face_vertex_mesh(&vertices, &faces)
}

static mut GLOBAL_SCENE: Option<Rc<RefCell<Scene>>> = None;

fn get_scene() -> &'static mut Rc<RefCell<Scene>> {
    unsafe { GLOBAL_SCENE.as_mut().unwrap() }
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
        // gl.enable(GL::CULL_FACE);

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

            camera_dist: 5.0,
            camera_pitch: 0.0,
            camera_yaw: 0.0,

            mesh: HalfEdgeMesh::empty(),
            // mesh: make_model_implicit(1., 10, |p| {
            //     p.to_vec().magnitude() - 0.5
            // }),
        };

        r.update();

        Ok(r)
    }

    fn update_mesh(&mut self, split: usize) {
        self.mesh = make_model_implicit(1., split, |p| {
            (Point3::from_vec(Vector3::new(p.x, 0.0, p.z).normalize_to(0.5)) - p).magnitude() - 0.2
        });
        self.update();
    }

    fn update(&mut self) {
        let mut verts = std::collections::HashMap::<u32, u16>::new();

        let mut v = Vec::new();

        for (i, (id, vert)) in self.mesh.vertices.iter().enumerate() {
            let vert = vert.borrow();
            v.push(vert.pos.x);
            v.push(vert.pos.y);
            v.push(vert.pos.z);
            verts.insert(*id, i as u16);
        }

        let c_e = [1.0, 0.0, 0.0, 1.0].repeat(verts.len());
        let i_e = self
            .mesh
            .edges
            .iter()
            .flat_map(|(_, edge)| {
                let edge = edge.borrow();

                let v0 = edge.get_origin().map(|v| v.borrow().id);

                let v1 = edge
                    .get_pair()
                    .and_then(|e| e.borrow().origin.upgrade())
                    .map(|v| v.borrow().id);

                if let (Some(v0), Some(v1)) = (v0, v1) {
                    vec![verts[&v0], verts[&v1]]
                } else {
                    vec![]
                }
            })
            .collect::<Vec<_>>();

        let c_f = [0.8, 1.0, 0.8, 1.0].repeat(verts.len());

        let i_f = self
            .mesh
            .faces
            .iter()
            .flat_map(|(_, face)| {
                let face = face.borrow();
                let verts = face
                    .adjacent_verts()
                    .filter_map(|v| v.upgrade().map(|v| verts[&v.borrow().id]))
                    .collect::<Vec<_>>();

                verts[1..]
                    .windows(2)
                    .flat_map(|w| vec![verts[0], w[1], w[0]])
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<u16>>();

        self.vao_tri.send_data(&v, &c_f, &i_f);
        self.vao_lin.send_data(&v, &c_e, &i_e);
    }

    fn draw(&self) {
        self.gl.use_program(Some(&self.program));
        send_mvp_matrix(
            &self.gl,
            &self.mvp_location,
            self.camera_pitch,
            self.camera_yaw,
            self.camera_dist,
        );

        self.gl.clear_color(0.0, 0.0, 0.0, 1.0);
        self.gl.clear_depth(1.0);
        self.gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        self.vao_tri.draw_elements(GL::TRIANGLES);
        self.vao_lin.draw_elements(GL::LINES);
        self.gl.flush();
    }

    fn move_camera(&mut self, pan: (f32, f32), zoom: f32) {
        self.camera_yaw -= pan.0 * 0.01;
        self.camera_pitch -= pan.1 * 0.01;

        self.camera_pitch = self
            .camera_pitch
            .clamp(-consts::FRAC_PI_2 + 1e-6, consts::FRAC_PI_2 - 1e-6);

        self.camera_dist *= 2.0f32.powf(zoom);
        self.camera_dist = self.camera_dist.clamp(1., 100.);

        self.update();
    }
}

const CANVAS_SIZE: u32 = 1024;

#[wasm_bindgen]
pub fn start(canvas: &HtmlCanvasElement) -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let scene = Rc::new(RefCell::new(Scene::new(canvas)?));

    unsafe {
        GLOBAL_SCENE = Some(scene);
    }

    Ok(())
}

#[wasm_bindgen]
pub fn draw() -> Result<(), JsValue> {
    let scene = get_scene();
    scene.borrow().draw();
    Ok(())
}

#[wasm_bindgen]
pub fn pan(dx: f32, dy: f32, zoom: f32) -> Result<(), JsValue> {
    let scene = get_scene();
    scene.borrow_mut().move_camera((dx, dy), zoom);
    Ok(())
}

#[wasm_bindgen]
pub fn update_spl(split: usize) -> Result<(), JsValue> {
    let scene = get_scene();
    scene.borrow_mut().update_mesh(split);
    Ok(())
}

fn send_mvp_matrix(
    gl: &GL,
    location: &WebGlUniformLocation,
    camera_pitch: f32,
    camera_yaw: f32,
    camera_dist: f32,
) {
    let eye = glm::quat_rotate_vec3(
        &(glm::quat_angle_axis(camera_yaw, &glm::Vec3::new(0.0, 1.0, 0.0))
            * glm::quat_angle_axis(camera_pitch, &glm::Vec3::new(1.0, 0.0, 0.0))),
        &glm::Vec3::new(0.0, 0.0, camera_dist),
    );
    let center = glm::Vec3::new(0.0, 0.0, 0.0);
    let up = glm::Vec3::new(0.0, 1.0, 0.0);
    let view_matrix = glm::look_at(&eye, &center, &up);

    let aspect = 1.0;
    let fovy = 45.0 * consts::PI / 180.0;
    let near = 0.1;
    let far = 100.0;
    let projection_matrix = glm::perspective(aspect, fovy, near, far);

    let mvp_matrix = projection_matrix * view_matrix;
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
