use crate::{error_fmt, EngineError::ElementArrayErr};
use std::{mem, ptr::addr_of_mut, rc::Rc};

#[derive(Clone, Debug)]
struct ElementArrayBufferInternal {
    id: u32,
    len: i32,
}

impl Drop for ElementArrayBufferInternal {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

/// Tells OpenGL in which order the vertices of the VertexBuffer should be drawn.
/// Internally (inside GPU memory) is an array of u32.
/// Three u32 in a row form one triangle primitive.
#[derive(Clone, Debug)]
pub struct ElementArrayBuffer(Rc<ElementArrayBufferInternal>);

impl ElementArrayBuffer {
    /// Create new `ElementArrayBuffer`. There is no builder pattern for this type.
    /// # Errors
    /// Returns error if the EAB object length exceeds an i32
    pub(crate) fn new(contents: &[u32]) -> crate::Result<Self> {
        let len = contents.len().try_into().map_err(|_| {
            ElementArrayErr(error_fmt!(ElementArrayBuffer, "EAB length exceeds i32"))
        })?;

        let id = unsafe {
            let mut id = 0;
            gl::GenBuffers(1, addr_of_mut!(id));
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                mem::size_of_val(contents).try_into().map_err(|_| {
                    ElementArrayErr(error_fmt!(ElementArrayBuffer, "EAB size exceeds isize"))
                })?,
                contents.as_ptr().cast(),
                gl::STATIC_DRAW,
            );
            id
        };

        Ok(Self(Rc::new(ElementArrayBufferInternal {
            id,
            len,
        })))
    }

    /// Get the id for use in gl functions
    pub(crate) fn id(&self) -> u32 {
        self.0.id
    }

    /// Get the length of the buffer. No `is_empty()` is needed.
    #[allow(clippy::len_without_is_empty)]
    pub(crate) fn len(&self) -> i32 {
        self.0.len
    }
}
