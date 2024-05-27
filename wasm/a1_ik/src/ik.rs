use nalgebra_glm::{angle, rotate_vec2, Vec2};
use web_sys::console;

pub const ORIGIN: Vec2 = Vec2::new(-0.5, 0.0);

pub struct IKArm {
    pub length: f32,
    pub angle: f32,
}

pub struct IK {
    pub arms: Vec<IKArm>,
}

fn angle_signed(v1: &Vec2, v2: &Vec2) -> f32 {
    let a = angle(v1, v2);
    if v1.x * v2.y - v1.y * v2.x < 0. {
        -a
    } else {
        a
    }
}

impl IK {
    pub fn new() -> Self {
        Self {
            arms: vec![
                IKArm {
                    length: 0.3,
                    angle: 0.0,
                },
                IKArm {
                    length: 0.3,
                    angle: 0.0,
                },
            ],
        }
    }

    pub fn render(&self) -> Vec<Vec2> {
        let mut points = vec![ORIGIN];
        let mut ang = 0.0;
        for arm in &self.arms {
            let last = points.last().unwrap();
            ang += arm.angle;
            points.push(last + rotate_vec2(&Vec2::new(arm.length, 0.), ang));
        }

        points
    }

    fn update_step(&mut self, target: Vec2) -> f32 {
        let points = self.render();
        let mut end = *points.last().unwrap();
        points[..points.len() - 1]
            .iter()
            .enumerate()
            .rev()
            .for_each(|(i, p)| {
                let d = end - p;
                let t = target - p;
                let a = angle_signed(&d, &t) / 1.0;
                self.arms[i].angle += a;
                let dd = rotate_vec2(&d, a);
                end = p + dd;
            });
        (points.last().unwrap() - end).magnitude()
    }

    pub fn update(&mut self, target: Vec2) {
        for _ in 0..100 {
            let d = self.update_step(target);
            if d < 1e-4 {
                break;
            }
        }
    }

    pub fn add_arm(&mut self) {
        self.arms.push(IKArm {
            length: 0.3,
            angle: 0.0,
        });
    }

    pub fn pop_arm(&mut self) {
        if self.arms.len() <= 1 { return; }
        self.arms.pop();
    }
}
