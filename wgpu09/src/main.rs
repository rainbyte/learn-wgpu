mod common;
mod vertex_data;
mod transforms;

fn vertex(p: [i8; 3], c: [i8; 3]) -> common::Vertex {
    common::Vertex {
        position: [
            p[0] as f32,
            p[1] as f32,
            p[2] as f32,
            1.0
        ],
        color: [
            c[0] as f32,
            c[1] as f32,
            c[2] as f32,
            1.0
        ],
    }
}

fn create_vertices() -> Vec<common::Vertex> {
    let pos = vertex_data::cube_positions();
    let col = vertex_data::cube_colors();
    let mut data: Vec<common::Vertex> = Vec::with_capacity(pos.len());
    for i in 0..pos.len() {
        data.push(vertex(pos[i], col[i]));
    }
    data.to_vec()
}

fn main() {
    let vertex_data = create_vertices();
    common::run(&vertex_data, "Cube with distinct face colors");
}
