struct Uniforms {
    mvpMatrix: mat4x4<f32>,
}
@binding(0) @group(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(@location(0) pos: vec4<f32>, @location(1) color: vec4<f32>) -> VertexOutput {
    var output: VertexOutput;
    output.position = uniforms.mvpMatrix * pos;
    output.color = color;
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}