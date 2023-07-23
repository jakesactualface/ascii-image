use std::{borrow::Cow, fmt};

mod scaler;
pub use crate::scaler::scale;

pub struct AsciiImage<'a> {
    pub width: usize,
    pub height: usize,
    pub data: Cow<'a, [u8]>,
}

impl fmt::Display for AsciiImage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.data.chunks_exact(self.width) {
            writeln!(f, "{:?}", chunk)?;
        }
        return Ok(());
    }
}
