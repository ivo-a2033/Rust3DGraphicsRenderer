use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title: "Game".into(),
        window_width: 800,
        window_height: 800,
        ..Default::default()
    }
}

// Id usually call a lib but this is fine too
struct Vec4 {
    x: f32,
    y: f32,
    z: f32,
    w: f32,

    
}

impl Vec4 {
    fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

struct Mat4 {
    m: [[f32; 4]; 4],
}

impl Mat4 {
    fn mul_vec4(&self, v: Vec4) -> Vec4 {
        Vec4 {
            x: self.m[0][0] * v.x
             + self.m[0][1] * v.y
             + self.m[0][2] * v.z
             + self.m[0][3] * v.w,

            y: self.m[1][0] * v.x
             + self.m[1][1] * v.y
             + self.m[1][2] * v.z
             + self.m[1][3] * v.w,

            z: self.m[2][0] * v.x
             + self.m[2][1] * v.y
             + self.m[2][2] * v.z
             + self.m[2][3] * v.w,

            w: self.m[3][0] * v.x
             + self.m[3][1] * v.y
             + self.m[3][2] * v.z
             + self.m[3][3] * v.w,
        }
    }
}

fn get_proj_mat(n: f32, fov: f32, aspect: f32) -> Mat4{

    let f = 1.0 / ((fov / 2.0) as f32).tan();
    let far = 100.0;
    let mat = Mat4 {
        m: [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + n) / (n - far), 2.0 * far * n / (n - far)],
            [0.0, 0.0, -1.0, 0.0],
        ],
    }; // proj matrix archetype

    return mat;
}

fn scale_matrix(sx: f32, sy: f32, sz: f32) -> Mat4 {
    Mat4 {
        m: [
            [sx, 0.0, 0.0, 0.0],
            [0.0, sy, 0.0, 0.0],
            [0.0, 0.0, sz, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    }
}

fn rotate_axis_matrix(axis: (f32, f32, f32), angle: f32) -> Mat4 {
    let (x, y, z) = axis; // must be normalized (length 1)
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;

    Mat4 {
        m: [
            [t*x*x + c,   t*x*y - s*z, t*x*z + s*y, 0.0],
            [t*x*y + s*z, t*y*y + c,   t*y*z - s*x, 0.0],
            [t*x*z - s*y, t*y*z + s*x, t*z*z + c,   0.0],
            [0.0,         0.0,         0.0,         1.0],
        ],
    }
}

fn to_screen(x: f32, y: f32) -> Vec2 {
    Vec2::new(
        (x + 1.0) * 400.0,
        (1.0 - y) * 400.0,
    )
}

fn get_projected(mut projected_points: Vec<[f32; 3]>, positions: &Vec<f32>, rotationy: f32) -> Vec<[f32; 3]> {
    projected_points.clear();
    for i in 0..positions.len()/3 {
        let v = i*3;
        let mut point = Vec4::new(positions[v], positions[v+1] , positions[v+2], 1.0);
        let scale = scale_matrix(0.0001, 0.0001, 0.0001);
        point = scale.mul_vec4(point);
        let rot_y = rotate_axis_matrix((0.0,1.0,0.0),rotationy);
        point = rot_y.mul_vec4(point);

        let rot_x = rotate_axis_matrix((1.0,0.0,0.0),-rotationy*2.0);
        point = rot_x.mul_vec4(point);


        point.z -= 0.001;
        let projection = get_proj_mat(1.0,  90f32.to_radians(), 1.0);

        let mut p = projection.mul_vec4(point);


        let ndc_x = p.x / p.w;
        let ndc_y = p.y / p.w;
        let ndc_z = p.z / p.w;

        let proj_point = [ndc_x, ndc_y, ndc_z];
        projected_points.push(proj_point);
    }

    return projected_points;
}

fn render(projected_points: Vec<[f32; 3]>, indices: &Vec<u32>){
    for i in 0..indices.len()/3 {
        let v = (i*3) as usize;
        let mut p1 = Vec2::new(projected_points[indices[v] as usize][0], projected_points[indices[v] as usize][1]);
        let mut p2 = Vec2::new(projected_points[indices[v+1] as usize][0], projected_points[indices[v+1] as usize][1]);
        let mut p3 = Vec2::new(projected_points[indices[v+2] as usize][0], projected_points[indices[v+2] as usize][1]);

        p1 = to_screen(p1.x, p1.y);
        p2 = to_screen(p2.x, p2.y);
        p3 = to_screen(p3.x, p3.y);

        draw_triangle(p1,p2,p3, Color::from_rgba(255,255,255,255));
    }
}



#[macroquad::main(window_conf)]
async fn main() {

    let (raw_models, _materials) = tobj::load_obj(
        "copyrightFreeToaster.obj",
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        }, 
       )
    .unwrap();

    let mut positions: Vec<f32> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for m in &raw_models {
        let index_offset = (positions.len() / 3) as u32;
        positions.extend_from_slice(&m.mesh.positions);
        indices.extend(m.mesh.indices.iter().map(|i| i + index_offset));
    }

    println!("vertices: {}", positions.len() / 3);
    println!("triangles: {}", indices.len() / 3);

    let mut projected_points = Vec::new();
    let mut rotationy = 0.0;

    loop {
        clear_background(BLACK);

        rotationy += 0.01;

        projected_points = get_projected(projected_points, &positions, rotationy);

        render(projected_points.clone(), &indices);

        next_frame().await;
    }
}