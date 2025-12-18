fn main() {
    let instances = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
    let adapters = pollster::block_on(
        instances.enumerate_adapters(wgpu::Backends::all())
    );
    for adapter in adapters {
        println!("{:?}", adapter.get_info())
    }
}
