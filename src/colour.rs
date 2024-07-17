#[derive(Debug, Clone, Copy)]
pub struct ColourRGB([f32; 3]);

macro_rules! get_colour {
    ($letter:ident, $index:literal) => {
        #[inline]
        pub fn $letter(self) -> f32 {
            self.0[$index]
        }
    };
}

impl ColourRGB {
    #[inline]
    pub fn new(input: [f32; 3]) -> Self {
        Self(input)
    }

    get_colour!(r, 0);
    get_colour!(g, 1);
    get_colour!(b, 2);

    #[inline]
    pub fn as_array(self) -> [f32; 3] {
        self.0
    }
}
