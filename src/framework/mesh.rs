
use glium::{Vertex, Frame, Program, DrawParameters};
use glium::vertex::{PerInstance};
use glium::index::{PrimitiveType};
use glium::uniforms::{Uniforms};
use glium::backend::{Facade};
use glium::Surface;
use glium::texture::Texture2d;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Mesh<V: Vertex> {

    vertices: Vec<V>,

    indices: Option<Vec<u16>>,

    primitive_type: PrimitiveType,

}

impl<V: Vertex> Mesh<V> {

    pub fn new() -> Self {
        Mesh {
            vertices: Vec::new(),
            indices: Some(Vec::new()),
            primitive_type: PrimitiveType::TrianglesList,
        }
    }

    pub fn wrap(vertices: Vec<V>, indices: Vec<u16>, primitive_type: PrimitiveType) -> Self {
        Mesh {
            vertices,
            indices: Some(indices),
            primitive_type,
        }
    }

    pub fn wrap_noind(vertices: Vec<V>, primitive_type: PrimitiveType) -> Self {
        Mesh {
            vertices,
            indices: None,
            primitive_type,
        }
    }

    pub fn push(&mut self, vertices: &[V], indices: &[u16]) -> bool {
        if let Some(ind) = &mut self.indices {
            if indices.len() >= vertices.len() {
                let vert = &mut self.vertices;
                let offset = vert.len() as u16;
                let start = ind.len() as usize;
                ind.extend_from_slice(indices);
                vert.extend_from_slice(vertices);               
                for i in &mut ind[start..] {
                    *i += offset;
                }              
                return true;              
            }
        }
        return false;
    }

    pub fn modify(&mut self) -> &mut [V] {
        self.vertices.as_mut_slice()
    }

    pub fn draw<U: Uniforms>(&self, facade: &dyn Facade, target: &mut Frame, program: &Program, uniforms: &U, draw_parameters: &DrawParameters) -> Result<&Self> {
        if !self.vertices.is_empty() {
            let vbo = glium::VertexBuffer::new(facade, self.vertices.as_slice()).map_err(Box::new)?;
            if let Some(indices) = &self.indices {
                let ind = glium::index::IndexBuffer::new(facade, self.primitive_type, indices.as_slice()).map_err(Box::new)?;
                target.draw(&vbo, &ind, program, uniforms, draw_parameters).map_err(Box::new)?;
            } else {
                let ind = glium::index::NoIndices(self.primitive_type);
                target.draw(&vbo, &ind, program, uniforms, draw_parameters).map_err(Box::new)?;
            }           
        }
        Ok(self)
    }

    pub fn draw_instances<U: Uniforms>(&self, facade: &dyn Facade, target: &mut Frame, per_instance: PerInstance, program: &Program, uniforms: &U, draw_parameters: &DrawParameters) -> Result<&Self> {
        if !self.vertices.is_empty() {
            let vbo = glium::VertexBuffer::new(facade, self.vertices.as_slice()).map_err(Box::new)?;
            if let Some(indices) = &self.indices {
                let ind = glium::index::IndexBuffer::new(facade, self.primitive_type, indices.as_slice()).map_err(Box::new)?;
                target.draw((&vbo, per_instance), &ind, program, uniforms, draw_parameters).map_err(Box::new)?;
            } else {
                let ind = glium::index::NoIndices(self.primitive_type);
                target.draw((&vbo, per_instance), &ind, program, uniforms, draw_parameters).map_err(Box::new)?;
            }           
        }
        Ok(self)
    }
}

pub const INDICES3_TRIANGLE: [u16;3] = [0,1,2];
pub const INDICES4_RECT: [u16;6] = [0,1,3,3,2,0];
pub const INDICES8_BLOCK: [u16;36] = [0,1,3,3,2,0, 0,1,5,5,4,0, 0,4,6,6,2,0, 1,5,7,7,3,1, 2,3,7,7,6,2, 4,5,7,7,6,4];

use std::io;
use std::io::Read;
use png;
use glium::texture::RawImage2d;

pub fn load_texture2d<R: Read>(ifile: R) -> io::Result<RawImage2d<'static, u8>> {
    let decoder = png::Decoder::new(ifile);
    let (info, mut reader) = decoder.read_info().map_err(|e| {io::Error::new(io::ErrorKind::InvalidInput, e)})?;
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf).unwrap();
    let tx = match info.color_type {
        png::ColorType::RGB => glium::texture::RawImage2d::from_raw_rgb(buf, (info.width, info.height)),
        png::ColorType::RGBA => glium::texture::RawImage2d::from_raw_rgba(buf, (info.width, info.height)),
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("{:?}", info.color_type)))   
    };
    Ok(tx)
}