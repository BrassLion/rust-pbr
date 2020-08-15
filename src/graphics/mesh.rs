use specs::prelude::*;

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
}

impl Component for Mesh {
    type Storage = VecStorage<Self>;
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

    pub fn new_from_glb(device: &wgpu::Device, glb_data: &[u8]) -> Self {
        let (gltf, buffers, images) = gltf::import_slice(glb_data.as_ref()).unwrap();

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let pos_iter = reader.read_positions().unwrap();
                let norm_iter = reader.read_normals().unwrap();
                // Read vertices and indices.
                for (vert_pos, vert_norm) in pos_iter.zip(norm_iter) {
                    vertices.push(Vertex {
                        position: vert_pos,
                        normal: vert_norm,
                    });
                }

                // Read indices.
                if let Some(iter) = reader.read_indices() {
                    for vertex_index in iter.into_u32() {
                        indices.push(vertex_index);
                    }
                }
            }
        }

        Mesh::new(device, vertices.as_slice(), Some(indices.as_slice()))
    }
}
