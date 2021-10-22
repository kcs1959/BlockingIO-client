use std::ops::{Add, Neg};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct BlockUnit(f32);

impl BlockUnit {
    pub fn into_render_value(&self) -> f32 {
        self.0 as f32 * 0.5
    }
}

impl From<i32> for BlockUnit {
    fn from(i: i32) -> Self {
        Self(i as f32)
    }
}

impl From<f32> for BlockUnit {
    fn from(f: f32) -> Self {
        Self(f)
    }
}

impl Add for BlockUnit {
    type Output = BlockUnit;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Neg for BlockUnit {
    type Output = BlockUnit;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}
