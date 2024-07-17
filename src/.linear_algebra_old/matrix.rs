use std::mem;

#[derive(Debug, Clone, Copy)]
pub struct Matrix<const ROW: usize, const COL: usize>([[f32; COL]; ROW]); // Col-Major!!!!! 

impl<const ROW: usize, const COL: usize> Default for Matrix<ROW, COL> {
    fn default() -> Self {
        Matrix([[0.0; COL]; ROW])
    }
}

impl<const ROW: usize, const COL: usize> Matrix<ROW, COL> {
    pub fn from_row_major(input: [[f32; ROW]; COL]) -> Self {
        Matrix(input).transpose()
    }

    pub fn col_major(&self) -> [f32; COL * ROW]{
        unsafe {
            mem::transmute_copy(self)
        }
    }
    
    pub fn zeros() -> Self {
        Self::default()
    }

    pub fn identity() -> Self {
        let mut output = Self::zeros();
        for i in 0..usize::min(ROW, COL) {
            output.0[i][i] = 1.0;
        }
        output
    }

    pub fn transpose(self) -> Matrix<COL, ROW> {
        let mut output = Matrix::<COL, ROW>::zeros();
        for i in 0..COL {
            for j in 0..ROW {
                output.0[i][j] = self.0[j][i]
            }
        }

        output
    }
}

impl Matrix<4, 4> {
    pub fn scale(sx: f32, sy: f32, sz: f32) -> Self {
        let mut matrix = Self::zeros();
        matrix.0[0][0] = sx;
        matrix.0[1][1] = sy;
        matrix.0[2][2] = sz;
        matrix.0[3][3] = 1.0;
        matrix
    }

    pub fn translate(x: f32, y: f32, z: f32) -> Self {
        let mut matrix = Self::identity();
        matrix.0[3][0] = x;
        matrix.0[3][1] = y;
        matrix.0[3][2] = z;
        matrix
    }

    pub fn perspective_matrix(fov: f32, aspect: f32, near: f32, far: f32) -> Self {
        let mut matrix = Self::zeros();
        matrix.0[0][0] = 1.0 / fov.tan() / aspect;
        matrix.0[1][1] = 1.0 / fov.tan();
        matrix.0[2][2] = -(far + near) / (far - near);
        matrix.0[2][3] = -1.0;
        matrix.0[3][2] = -2.0 * far * near / (far - near);
        matrix
    }
}
/*
impl<const ROW0COL1: usize, const COL0: usize, const ROW1: usize>
    Mul<Matrix<ROW1, ROW0COL1>> for Matrix<ROW0COL1, COL0> {
        type Output = Matrix<ROW1, COL0>;

        fn mul(self, rhs: Matrix<ROW1, ROW0COL1>) -> Self::Output {
            let output = Self::Output::zeros();
            
            
            output
        }
}*/

#[deprecated]
pub fn zeros() -> [f32; 16] {
    [0f32; 16]
}

#[deprecated]
pub fn identity() -> [f32; 16] {
    let mut matrix = zeros();
    matrix[0] = 1.0;
    matrix[5] = 1.0;
    matrix[10] = 1.0;
    matrix[15] = 1.0;
    matrix
}

#[deprecated]
pub fn translate(x: f32, y: f32, z: f32) -> [f32; 16] {
    let mut matrix = identity();
    matrix[12] = x;
    matrix[13] = y;
    matrix[14] = z;
    matrix
}


#[deprecated]
pub fn orthogonal_matrix(
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    near: f32,
    far: f32,
) -> [f32; 16] {
    let mut matrix = zeros();
    let w = right - left;
    let x = right + left;
    let h = top - bottom;
    let y = top + bottom;
    let d = far - near;
    let z = far + near;
    matrix[0] = 2.0 / w;
    matrix[5] = 2.0 / h;
    matrix[10] = -1.0 / d;
    matrix[12] = -x / w;
    matrix[13] = -y / h;
    matrix[14] = -z / d;
    matrix[15] = 1.0;
    matrix
}

#[deprecated]
pub fn perspective_matrix(fov: f32, aspect: f32, near: f32, far: f32) -> [f32; 16] {
    let mut matrix = zeros();
    matrix[0] = 1.0 / fov.tan() / aspect;
    matrix[5] = 1.0 / fov.tan();
    matrix[10] = -(far + near) / (far - near);
    matrix[11] = -1.0;
    matrix[14] = -2.0 * far * near / (far - near);
    matrix
}

pub fn matmul(a: [f32; 16], b: [f32; 16]) -> [f32; 16] {
    let mut c = zeros();
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                c[i * 4 + j] += a[i * 4 + k] * b[k * 4 + j];
            }
        }
    }
    c
}
