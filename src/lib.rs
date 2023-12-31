use std::{borrow::Cow, fmt};

mod scaler;
pub use crate::scaler::scale;

use lazy_static::lazy_static;

pub struct RectSize {
    pub width: usize,
    pub height: usize,
}

pub struct ImageData<'a> {
    pub width: usize,
    pub height: usize,
    pub data: Cow<'a, [u8]>,
}

impl fmt::Display for ImageData<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for chunk in self.data.chunks_exact(self.width) {
            for c in chunk {
                write!(f, "{}", get_ascii_character(c))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

lazy_static! {
    static ref CHARACTERS: Vec<char> = r#" .,:ilwW"#.chars().collect();
}

fn get_ascii_character(brightness: &u8) -> &'static char {
    let interpolated = (*brightness as f32) / (u8::MAX as f32);
    let index = interpolated * (CHARACTERS.len() - 1) as f32;
    let index = index.round() as usize;
    return CHARACTERS.get(index).unwrap();
}
