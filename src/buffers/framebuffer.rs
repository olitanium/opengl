use crate::{
    error_fmt, material::Material, texture::Texture, EngineError, EngineError::FrameBufferErr,
    Result,
};

/// A `FrameBuffer` is a destination for drawing a scene, the default FrameBuffer is accessible
/// after an `Environment` is initialzed and is for drawing to the screen. Any other framebuffers
/// will draw to a colour buffer (or HDR buffer) which can be retreived with
/// `FrameBuffer::get_colour`
pub struct FrameBuffer {
    id: u32,
    colour: InternalBufferColourType,
    stencilordepth: StencilOrDepth,
}

#[expect(dead_code)]
enum StencilOrDepth {
    DefaultFrameBuffer,
    DepthStencil(Texture),
    Depth(Texture),
    None,
}

impl FrameBuffer {
    /// Begin the build process for a `FrameBuffer`
    #[inline]
    pub fn builder() -> Builder {
        Builder::default()
    }

    // Internal function to generate the default (screen) `FrameBuffer`
    pub(crate) fn new_default() -> Self {
        Self {
            id: 0,
            colour: InternalBufferColourType::DefaultRgb,
            stencilordepth: StencilOrDepth::DefaultFrameBuffer,
        }
    }

    /// Bind the `FrameBuffer` for all subsequent draw calls.
    pub(crate) fn bind(&mut self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.id);
            match &self.stencilordepth {
                StencilOrDepth::DefaultFrameBuffer => {
                    gl::Enable(gl::DEPTH_TEST);
                    gl::Enable(gl::STENCIL_TEST);
                }
                StencilOrDepth::DepthStencil(_) => {
                    gl::Enable(gl::DEPTH_TEST);
                    gl::Enable(gl::STENCIL_TEST);
                }
                StencilOrDepth::Depth(_) => {
                    gl::Enable(gl::DEPTH_TEST);
                    gl::Disable(gl::STENCIL_TEST);
                }
                StencilOrDepth::None => {
                    gl::Disable(gl::DEPTH_TEST);
                    gl::Disable(gl::STENCIL_TEST);
                }
            }
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        }
    }

    #[inline]
    pub fn get_colour(&self) -> Result<Texture> {
        match &self.colour {
            InternalBufferColourType::TexRgb(x) => Ok(x.clone()),
            InternalBufferColourType::DefaultRgb => Err(FrameBufferErr(error_fmt!(
                frame_buffer::FrameBuffer,
                "Cannot get default framebuffer as a texture"
            ))),
        }
    }

    #[inline]
    pub fn as_material(&self) -> Result<Material> {
        Ok(Material::builder().diffuse(self.get_colour()?).build())
    }
}

impl Drop for FrameBuffer {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}

#[derive(Debug)]
pub enum BufferColourType {
    TexRgb,
}

enum InternalBufferColourType {
    DefaultRgb,
    TexRgb(Texture),
}

#[derive(Default, Debug)]
pub struct Builder {
    colour: Option<BufferColourType>,
    stencil: bool,
    depth: bool,
    width: i32,
    height: i32,
}

impl Builder {
    #[inline]
    pub fn add_colour(mut self, colour: BufferColourType) -> Self {
        self.colour = Some(colour);
        self
    }

    #[inline]
    pub fn add_stencil(mut self) -> Self {
        self.stencil = true;
        self
    }

    #[inline]
    pub fn add_depth(mut self) -> Self {
        self.depth = true;
        self
    }

    #[inline]
    pub fn add_dims(mut self, width: i32, height: i32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    #[inline]
    pub fn build(self) -> Result<FrameBuffer> {
        let id = unsafe {
            let mut id = 0;
            gl::CreateFramebuffers(1, &mut id);
            id
        };
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
        }

        let colour_enum = self.colour.unwrap_or(BufferColourType::TexRgb);
        let colour = match colour_enum {
            BufferColourType::TexRgb => {
                InternalBufferColourType::TexRgb(Texture::framebuffer_attachment(
                    gl::RGB as i32,
                    self.width,
                    self.height,
                    gl::RGB,
                    gl::UNSIGNED_BYTE,
                ))
            }
        };

        match &colour {
            InternalBufferColourType::TexRgb(texture) => unsafe {
                gl::FramebufferTexture2D(
                    gl::FRAMEBUFFER,
                    gl::COLOR_ATTACHMENT0,
                    gl::TEXTURE_2D,
                    texture.id(),
                    0,
                );
            },
            InternalBufferColourType::DefaultRgb => {}
        }

        // format, internal_format, type, attachment
        let stencilordepth = match (self.stencil, self.depth) {
            (true, true) => {
                let texture = Texture::framebuffer_attachment(
                    gl::DEPTH24_STENCIL8 as i32,
                    self.width,
                    self.height,
                    gl::DEPTH_STENCIL,
                    gl::UNSIGNED_INT_24_8,
                );
                unsafe {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::DEPTH_STENCIL_ATTACHMENT,
                        gl::TEXTURE_2D,
                        texture.id(),
                        0,
                    );
                }
                StencilOrDepth::DepthStencil(texture)
            }
            (false, true) => {
                let texture = Texture::framebuffer_attachment(
                    gl::DEPTH_COMPONENT as i32,
                    self.width,
                    self.height,
                    gl::DEPTH_COMPONENT,
                    gl::UNSIGNED_INT,
                );
                unsafe {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::DEPTH_ATTACHMENT,
                        gl::TEXTURE_2D,
                        texture.id(),
                        0,
                    );
                }
                StencilOrDepth::Depth(texture)
            }
            (true, false) => {
                return Err(EngineError::FrameBufferErr(error_fmt!(
                    FrameBufferBuilder,
                    "Making a texture with a stencil but no depth is illegal"
                )));
            }
            (false, false) => StencilOrDepth::None,
        };

        let out =
            if unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) } == gl::FRAMEBUFFER_COMPLETE {
                Ok(FrameBuffer {
                    id,
                    colour,
                    stencilordepth,
                })
            } else {
                Err(EngineError::FrameBufferErr(error_fmt!(
                    frame_buffer::FrameBufferBuilder,
                    "Framebuffer incomplete"
                )))
            };

        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        out
    }
}
