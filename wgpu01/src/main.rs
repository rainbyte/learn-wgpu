fn main() {
    let instances = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    for adapter in instances.enumerate_adapters(wgpu::Backends::all()) {
        println!("{:?}", adapter.get_info())
    }
}
