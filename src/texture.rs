use std::ptr;
use std::rc::Rc;

use crate::buffers::framebuffer::FrameBuffer;
use crate::EngineError::TextureErr;
use crate::{error_fmt, some_builder, Result};

#[derive(Debug)]
struct Internal {
    id: u32,
}

#[derive(Clone, Debug)]
pub struct Texture(Rc<Internal>);

#[derive(Default)]
pub struct Builder {
    image_data: Option<image::DynamicImage>,
    dims: Option<(i32, i32)>,
    not_normalised: bool,
    wrap_s: Option<u32>,
    wrap_t: Option<u32>,
    mag_filter: Option<u32>,
    min_filter: Option<u32>,
}

impl Builder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// # Errors
    ///
    #[inline]
    pub fn image(mut self, filepath: &str) -> Result<Self> {
        self.image_data = Some(
            image::io::Reader::open(filepath)
                .map_err(|_| {
                    TextureErr(error_fmt!(
                        texture::Builder,
                        "Opening texture at {filepath}"
                    ))
                })?
                .decode()
                .map_err(|_| {
                    TextureErr(error_fmt!(
                        texture::Builder,
                        "Error parsing open texture {filepath}"
                    ))
                })?,
        );
        Ok(self)
    }

    #[must_use]
    #[inline]
    pub const fn set_wrap_s_t(mut self, wrap_s: u32, wrap_t: u32) -> Self {
        self.wrap_s = Some(wrap_s);
        self.wrap_t = Some(wrap_t);
        self
    }

    some_builder!(min_filter: u32);
    some_builder!(mag_filter: u32);

    #[must_use]
    #[inline]
    pub const fn not_normalised(mut self) -> Self {
        self.not_normalised = true;
        self
    }

    #[must_use]
    #[inline]
    pub fn monochrome(mut self, colour: [f32; 4]) -> Self {
        let mut image = image::Rgba32FImage::new(1, 1);
        image.get_pixel_mut(0, 0).0 = colour;
        self.image_data = Some(image::DynamicImage::ImageRgba32F(image));
        self
    }

    /// # Errors
    ///
    #[inline]
    pub fn build(self) -> Result<Texture> {
        let mut output = Internal { id: 0 };
        unsafe {
            gl::GenTextures(1, ptr::addr_of_mut!(output.id));
            gl::BindTexture(gl::TEXTURE_2D, output.id);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                #[expect(clippy::cast_possible_wrap)]
                (self.wrap_s.unwrap_or(gl::REPEAT) as i32),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                #[expect(clippy::cast_possible_wrap)]
                (self.wrap_t.unwrap_or(gl::REPEAT) as i32),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                #[expect(clippy::cast_possible_wrap)]
                (self.min_filter.unwrap_or(gl::LINEAR) as i32),
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                #[expect(clippy::cast_possible_wrap)]
                (self.mag_filter.unwrap_or(gl::LINEAR) as i32),
            );

            if let Some(image_data) = self.image_data {
                if self.not_normalised {
                    let image = image_data.flipv().into_rgba32f().into_flat_samples();

                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        #[expect(clippy::cast_possible_wrap)]
                        (gl::RGBA16F as i32),
                        image.layout.width.try_into().map_err(|_| {
                            TextureErr(error_fmt!(texture::Builder, "Texture width exceeds i32"))
                        })?,
                        image.layout.height.try_into().map_err(|_| {
                            TextureErr(error_fmt!(texture::Builder, "Texture height exceeds i32"))
                        })?,
                        0,
                        gl::RGBA,
                        gl::FLOAT,
                        image.samples.as_ptr().cast(),
                    );
                } else {
                    // if NOT self.not_normalised
                    let image = image_data.flipv().into_rgba8().into_flat_samples();

                    gl::TexImage2D(
                        gl::TEXTURE_2D,
                        0,
                        #[expect(clippy::cast_possible_wrap)]
                        (gl::RGBA as i32),
                        image.layout.width.try_into().map_err(|_| {
                            TextureErr(error_fmt!(texture::Builder, "Texture width exceeds i32"))
                        })?,
                        image.layout.height.try_into().map_err(|_| {
                            TextureErr(error_fmt!(texture::Builder, "Texture height exceeds i32"))
                        })?,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        image.samples.as_ptr().cast(),
                    );
                }
            } else if let Some((width, height)) = self.dims {
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    #[expect(clippy::cast_possible_wrap)]
                    (gl::RGBA32F as i32),
                    width,
                    height,
                    0,
                    gl::RGBA,
                    gl::FLOAT,
                    ptr::null(),
                );
            } else {
                return Err(TextureErr(error_fmt!(
                    texture::Builder,
                    "No image data or image dimensions given"
                )));
            }

            if let Some(
                gl::LINEAR_MIPMAP_LINEAR
                | gl::LINEAR_MIPMAP_NEAREST
                | gl::NEAREST_MIPMAP_LINEAR
                | gl::NEAREST_MIPMAP_NEAREST,
            ) = self.min_filter
            {
                gl::GenerateMipmap(gl::TEXTURE_2D);
            }

            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(Texture(Rc::new(output)))
    }
}

impl Texture {
    #[must_use]
    #[inline]
    pub fn builder() -> Builder {
        Builder::new()
    }

    #[must_use]
    #[inline]
    pub fn blank() -> Self {
        Self::grayscale(0.0, 0.0)
    }

    #[must_use]
    #[inline]
    pub fn grayscale(colour: f32, alpha: f32) -> Self {
        Self::all_one_colour([colour, colour, colour, alpha])
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    #[inline]
    pub fn all_one_colour(colour: [f32; 4]) -> Self {
        Self::builder()
            .monochrome(colour)
            .build()
            .expect("Should never fail")
    }

    /// # Errors
    #[inline]
    pub fn from_framebuffer(fb: &FrameBuffer) -> Result<Self> {
        fb.get_colour()
    }

    pub(crate) fn framebuffer_attachment(
        internalformat: i32,
        width: i32,
        height: i32,
        format: u32,
        data_type: u32,
    ) -> Self {
        let id = unsafe {
            let mut id = 0;

            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            #[allow(clippy::cast_possible_wrap)]
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internalformat,
                width,
                height,
                0,
                format,
                data_type,
                ptr::null(),
            );

            gl::BindTexture(gl::TEXTURE_2D, 0);

            id
        };

        Self(Rc::new(Internal { id }))
    }

    #[must_use]
    #[inline]
    pub fn id(&self) -> u32 {
        self.0.id
    }

    /// # Errors
    #[inline]
    pub fn bind_to(&self, index: u32) -> Result<()> {
        if index < 16 { // TODO: Programmatic replacement to 16 here
            unsafe {
                gl::ActiveTexture(gl::TEXTURE0 + index);
                gl::BindTexture(gl::TEXTURE_2D, self.0.id);
            }
            Ok(())
        } else {
            Err(TextureErr(error_fmt!(
                texture::Builder,
                "Cannot bind to a texture index greater than {}", 16 - 1
            )))
        }
    }
}

impl Drop for Internal {
    fn drop(&mut self) {
        unsafe { gl::DeleteTextures(1, &self.id) }
    }
}

impl Default for Texture {
    #[inline]
    fn default() -> Self {
        Self::blank()
    }
}
