use specs::prelude::*;

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
}

impl Component for Mesh {
    type Storage = VecStorage<Self>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertex_data: &[Vertex]) -> Self {
        // Upload vertex data.
        let vertex_data_bytes = unsafe {
            let len = vertex_data.len() * std::mem::size_of::<Vertex>();
            let ptr = vertex_data.as_ptr() as *const u8;
            std::slice::from_raw_parts(ptr, len)
        };

        let vertex_buffer =
            device.create_buffer_with_data(vertex_data_bytes, wgpu::BufferUsage::VERTEX);

        Self {
            vertex_buffer: vertex_buffer,
            num_vertices: vertex_data.len() as u32,
        }
    }
}
