struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4<f32>(in.pos, 0.0, 1.0);
    output.color = vec4<f32>(in.color, 1.0);
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
