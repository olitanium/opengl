use crate::{
    buffers::{element_array_buffer::ElementArrayBuffer, vertex_buffer::VertexBuffer}, error_fmt, linear_algebra::multizip, EngineError::VertexArrayErr, Result
};

use std::{mem, ptr, rc::Rc, vec};

#[derive(Debug)]
pub struct VertexArray {
    id: u32,
    vertex_buffer: VertexBuffer,
    element_buffer: ElementArrayBuffer,
    labels: Rc<Vec<String>>,
    lengths: Rc<Vec<usize>>,
}

impl Drop for VertexArray {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, ptr::addr_of!(self.id));
        }
    }
}

impl VertexArray {
    #[inline]
    pub fn builder() -> Builder {
        Builder::default()
    }

    pub(crate) fn configure_strides(&self) -> crate::Result<()> {
        let mut pointer = ptr::null::<f32>();
        let vbo_stride = mem::size_of::<f32>() * self.lengths.iter().sum::<usize>();
        unsafe {
            gl::BindVertexArray(self.id);
            for (index, length) in self.lengths.iter().enumerate() {
                gl::VertexAttribPointer(
                    index.try_into().map_err(|_| {
                        VertexArrayErr(error_fmt!(VertexArray, "VBO length exceeds u32"))
                    })?,
                    (*length).try_into().map_err(|_| {
                        VertexArrayErr(error_fmt!(VertexArray, "Attribute length exceeds i32"))
                    })?,
                    gl::FLOAT,
                    gl::FALSE,
                    vbo_stride.try_into().map_err(|_| {
                        VertexArrayErr(error_fmt!(VertexArray, "Stride exceeds i32"))
                    })?,
                    pointer.cast(),
                );
                pointer = pointer.wrapping_add(*length);

                gl::EnableVertexAttribArray(index.try_into().map_err(|_| {
                    VertexArrayErr(error_fmt!(VertexArray, "VBO length exceeds u32"))
                })?);
            }
        }
        Ok(())
    }

    pub(crate) fn draw(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
            gl::DrawElements(
                gl::TRIANGLES,
                self.element_buffer.len(),
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }
}

impl Clone for VertexArray {
    #[inline]
    fn clone(&self) -> Self {
        let vao_id = unsafe {
            let mut vao_id = 0;
            gl::GenVertexArrays(1, ptr::addr_of_mut!(vao_id));
            gl::BindVertexArray(vao_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer.id());
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.element_buffer.id());
            vao_id
        };

        let output = Self {
            id: vao_id,
            vertex_buffer: self.vertex_buffer.clone(),
            element_buffer: self.element_buffer.clone(),
            labels: self.labels.clone(),
            lengths: self.lengths.clone(),
        };

        output
            .configure_strides()
            .expect("This has already run once, and should be correct");

        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        output
    }
}

#[derive(Default)]
pub struct Builder {
    vb_content: Vec<(String, Vec<Vec<f32>>, usize)>,
    eb_content: Vec<u32>,
}

impl Builder {
    #[inline]
    pub fn attribute(
        mut self,
        name: String,
        attrib_array: Vec<Vec<f32>>,
    ) -> Result<Self> {
        let attrib_len = attrib_array
            .first()
            .ok_or_else(|| {
                VertexArrayErr(error_fmt!(
                    vertex_array::Builder,
                    "Attribute {name} has no elements"
                ))
            })?
            .len();

        if attrib_len == 0 {
            return Err(VertexArrayErr(error_fmt!(
                vertex_array::Builder,
                "Attribute {name} has no length"
            )));
        }

        if !attrib_array.iter().all(|a| a.len() == attrib_len)
        {
            return Err(VertexArrayErr(error_fmt!(
                vertex_array::Builder,
                "Attribute {name} is not homogenous in vector length (all 2D, 3D etc)"
            )));
        }

        self.vb_content.push((name, attrib_array, attrib_len));

        Ok(self)
    }

    #[inline]
    pub fn element_buffer(mut self, buffer: Vec<u32>) -> Self {
        self.eb_content = buffer;
        self
    }

    /// # Errors
    #[inline]
    pub fn build(self) -> Result<VertexArray> {
        let num_vertices = self
            .vb_content
            .first()
            .ok_or_else(|| {
                VertexArrayErr(error_fmt!(
                    vertex_array::Builder,
                    "No attributes in Vertex Array"
                ))
            })?
            .1
            .len();

        if !self.vb_content.iter().all(|x| x.1.len() == num_vertices) {
            return Err(VertexArrayErr(error_fmt!(
                vertex_array::Builder,
                "Build error, not all attributes are the same length"
            )));
        }

        let id = unsafe {
            let mut id = 0;
            gl::GenVertexArrays(1, &mut id);
            gl::BindVertexArray(id);
            id
        };

        // Unzip
        let mut vbo_labels = Vec::with_capacity(self.vb_content.len());
        let mut vbo_attrib = Vec::with_capacity(self.vb_content.len());
        let mut vbo_length = Vec::with_capacity(self.vb_content.len());

        for (label, attrib, length) in self.vb_content {
            vbo_labels.push(label);
            vbo_attrib.push(attrib);
            vbo_length.push(length);
        }

        let vbo_data = {
            let mut vbo_data = Vec::new();
            let text = vbo_attrib
                .into_iter()
                .map(|x| x.into_iter())
                .collect::<Vec<_>>();

            let mz = multizip::Multizip(text);

            for vertex in mz {
                for attribute in vertex {
                    vbo_data.extend(attribute.into_iter());
                }
            }

            vbo_data
        };

        let vertex_buffer = VertexBuffer::new(&vbo_data)?;

        let element_buffer = {
            let ebo_data = if self.eb_content.is_empty() {
                (0..num_vertices.try_into().expect("num_vertices exceeds u32")).collect()
            } else {
                self.eb_content
            };

            ElementArrayBuffer::new(&ebo_data)
        }?;

        let output = VertexArray {
            id,
            vertex_buffer,
            element_buffer, // do with sharing buffer data, probably a lot of rubbish
            labels: Rc::new(vbo_labels),
            lengths: Rc::new(vbo_length),
        };

        let _ = output.configure_strides();

        unsafe {
            gl::BindVertexArray(0);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }

        Ok(output)
    }
}

impl VertexArray {
    pub fn cube(side_length: f32) -> Result<Vec<Self>> {  
        let vertices: Vec<Vec<f32>> = vec![
               // x    y    z
            vec![0.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 1.0, 1.0],
            vec![1.0, 0.0, 0.0],
            vec![1.0, 0.0, 1.0],
            vec![1.0, 1.0, 0.0],
            vec![1.0, 1.0, 1.0f32],
        ]
        .into_iter().map(|face| face.into_iter().map(|elem| (elem - 0.5) * side_length).collect()).collect();

        #[rustfmt::ignore]
        let face_pos_normals = vec![
            (vec![5, 4, 6, 7], vec![1.0, 0.0, 0.0]    ),
            (vec![0, 1, 3, 2], vec![-1.0, 0.0, 0.0]   ),
            (vec![2, 3, 7, 6], vec![0.0, 1.0, 0.0]    ),
            (vec![1, 0, 4, 5], vec![0.0, -1.0, 0.0]   ),
            (vec![1, 5, 7, 3], vec![0.0, 0.0, 1.0]    ),
            (vec![4, 0, 2, 6], vec![0.0, 0.0, -1.0f32]),
        ];

        face_pos_normals
            .into_iter()
            .map(|(face, normal)| {
                VertexArray::builder()
                    .attribute(
                        "location".into(),
                        face.iter().copied().map(|index| vertices[index].clone()).collect(),
                    )?
                    .attribute(
                        "texture".into(),
                        vec![
                            vec![0.0, 0.0],
                            vec![1.0, 0.0],
                            vec![1.0, 1.0],
                            vec![0.0, 1.0],
                        ],
                    )?
                    .attribute("normal".into(), vec::from_elem(normal.to_vec(), 4))?
                    .element_buffer(vec![0, 1, 2, 0, 2, 3])
                    .build()
            })
            .collect()
    }
}
