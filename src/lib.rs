#![expect(incomplete_features)]
#![feature(array_windows)]
#![feature(stmt_expr_attributes)]
#![feature(generic_const_exprs)]
#![feature(box_into_inner)]

#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::nursery)]
//#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::restriction)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]


#![warn(clippy::semicolon_inside_block)]
#![allow(clippy::semicolon_outside_block)]
#![warn(clippy::semicolon_if_nothing_returned)]

#![expect(clippy::implicit_return)]
#![expect(clippy::as_conversions)]
#![expect(clippy::undocumented_unsafe_blocks)]
#![expect(clippy::float_arithmetic)]
#![expect(clippy::missing_const_for_fn)]
#![expect(clippy::must_use_candidate)]
#![expect(clippy::single_call_fn)]

pub mod buffers;
pub mod camera;
pub mod colour;
pub mod drawing;
pub mod environment;
pub mod global_state;
pub mod input;
pub mod lighting;
pub mod linear_algebra;
pub mod material;
pub mod modelling;
pub mod shader_program;
pub mod texture;
pub mod window;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum EngineError {
    GlfwErr(String),
    TextureErr(String),
    VectorErr(String),
    FrameBufferErr(String),
    ShaderErr(String),
    ElementArrayErr(String),
    VertexArrayErr(String),
    VertexBufferErr(String),
    MiscErr(String),
}

use core::result;

pub type Result<T> = result::Result<T, EngineError>;

#[macro_export]
macro_rules! getter {
    ($value:ident, $type:ty) => {
        #[must_use]
        #[inline]
        pub fn $value(&self) -> &$type {
            &self.$value
        }
    };
}

#[macro_export]
macro_rules! getter_clone {
    ($value:ident, $type:ty) => {
        #[must_use]
        #[inline]
        pub fn $value(&self) -> $type {
            self.$value.clone()
        }
    };
}

#[macro_export]
macro_rules! error_fmt {
    ($t:path, $($arg:tt)*) => {
        format!("{} error: {}", stringify!($t), format!($($arg)*))
    };
}

#[macro_export]
macro_rules! some_builder {
    ($name:ident: $type:ty) => {
        #[must_use]
        #[inline]
        pub fn $name(mut self, $name: $type) -> Self {
                self.$name = Some($name);
                self
        }
    };
}
