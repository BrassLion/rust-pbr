pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: Option<wgpu::Buffer>,
    pub num_vertices: u32,
    pub num_indices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tangent: [f32; 4], // tangent vector + bitangent sign.
    pub tex_coord: [f32; 2],
}

impl Mesh {
    pub fn new(device: &wgpu::Device, vertex_data: &[Vertex], index_data: Option<&[u32]>) -> Self {
        // Upload vertex data.
        let vertex_data_bytes = unsafe {
            let len = vertex_data.len() * std::mem::size_of::<Vertex>();
            let ptr = vertex_data.as_ptr() as *const u8;
            std::slice::from_raw_parts(ptr, len)
        };

        let vertex_buffer =
            device.create_buffer_with_data(vertex_data_bytes, wgpu::BufferUsage::VERTEX);

        // Upload index buffer if it exists.
        let index_buffer = match index_data {
            None => None,
            Some(data) => {
                let index_data_bytes = unsafe {
                    let len = data.len() * std::mem::size_of::<u32>();
                    let ptr = data.as_ptr() as *const u8;
                    std::slice::from_raw_parts(ptr, len)
                };

                let buffer =
                    device.create_buffer_with_data(index_data_bytes, wgpu::BufferUsage::INDEX);

                Some(buffer)
            }
        };

        let num_indices = match index_data {
            None => 0,
            Some(data) => data.len(),
        };

        Self {
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            num_vertices: vertex_data.len() as u32,
            num_indices: num_indices as u32,
        }
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_vertex_buffer(0, &self.vertex_buffer, 0, 0);

        match &self.index_buffer {
            Some(index_buffer) => {
                render_pass.set_index_buffer(&index_buffer, 0, 0);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
            None => {
                render_pass.draw(0..self.num_vertices, 0..1);
            }
        }
    }
}
