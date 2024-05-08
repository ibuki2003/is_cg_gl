use nalgebra_glm::{Vec2, Vec3};

#[derive(Clone, Copy)]
pub enum CatmullRomParmType {
    Uniform,
    ChordLength,
    Centripetal,
}

#[derive(Clone, Copy)]
pub enum CurveType {
    Bezier,
    CatmullRom(CatmullRomParmType),
}


fn make_bezier_normal(points: &[Vec2], n: usize) -> Vec<Vec2> {
    let m = points.len();
    (0..n + 1)
        .map(|i| {
            let t = i as f32 / n as f32;
            let mut v = points.to_vec();
            for j in (1..m).rev() {
                for k in 0..j {
                    v[k] = v[k] * (1. - t) + v[k + 1] * t;
                }
            }
            v[0]
        })
        .collect()
}

fn make_catmull_rom(points: &[Vec2], n: usize, curvetype: CatmullRomParmType) -> Vec<Vec2> {
    let m = points.len();

    let mut segments = points.iter().map(|w| Vec3::new(w.x, w.y, 0.0))
        .collect::<Vec<_>>();

    for i in 1..m {
        let d = match curvetype {
            CatmullRomParmType::Uniform => 1.0,
            CatmullRomParmType::ChordLength => (points[i] - points[i - 1]).norm(),
            CatmullRomParmType::Centripetal => (points[i] - points[i - 1]).norm().sqrt(),
        };
        segments[i].z = segments[i - 1].z + d;
    }

    let sum_len = segments[m - 1].z;

    let mut idx = 0;
    (0..n + 1)
        .map(|i| {
            let t = i as f32 / n as f32 * sum_len;
            while idx + 2 < m && t > segments[idx + 1].z {
                idx += 1;
            }

            interp(&segments[idx.max(1)-1..(idx + 3).min(m)], t)
        })
        .collect()
}

fn interp(p: &[Vec3], z: f32) -> Vec2 {
    match p.len() {
        1 => p[0].xy(),
        2 => {
            let t = (z - p[0].z) / (p[1].z - p[0].z);
            p[0].xy() * (1. - t) + p[1].xy() * t
        },
        3 => {
            let p0 = interp(&p[0..2], z);
            let p0 = Vec3::new(p0.x, p0.y, p[0].z);

            let p1 = interp(&p[1..3], z);
            let p1 = Vec3::new(p1.x, p1.y, p[2].z);

            interp(&[p0, p1], z)
        },
        4 => {
            let p0 = interp(&p[0..3], z);
            let p0 = Vec3::new(p0.x, p0.y, p[1].z);

            let p1 = interp(&p[1..4], z);
            let p1 = Vec3::new(p1.x, p1.y, p[2].z);

            interp(&[p0, p1], z)
        },
        _ => unimplemented!(),
    }
}

pub fn make_curve(points: &[Vec2], n: usize, curvetype: CurveType) -> Vec<Vec2> {
    match curvetype {
        CurveType::Bezier => make_bezier_normal(points, n),
        CurveType::CatmullRom(t) => make_catmull_rom(points, n, t),
    }
}
