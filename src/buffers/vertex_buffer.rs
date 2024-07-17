use crate::{error_fmt, EngineError::VertexBufferErr};
use std::{mem, sync::Arc};

#[derive(Debug)] // No Clone
pub struct VertexBufferInternal {
    id: u32,
}

#[derive(Clone, Debug)]
pub struct VertexBuffer(Arc<VertexBufferInternal>);

impl VertexBuffer {
    /// # Errors
    pub fn new(contents: &[f32]) -> crate::Result<Self> {
        let id = unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of_val(contents)).try_into().map_err(|_| {
                    VertexBufferErr(error_fmt!(VertexBuffer, "VBO length exceeds i32"))
                })?,
                contents.as_ptr().cast(),
                gl::STATIC_DRAW,
            );

            id
        };

        Ok(Self(Arc::new(VertexBufferInternal { id })))
    }

    pub fn id(&self) -> u32 {
        self.0.id
    }
}

impl Drop for VertexBufferInternal {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}