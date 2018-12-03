use std::fmt::Display;

pub use markup_proc_macro::define;

pub trait Render {
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<'a, T: Render + ?Sized> Render for &'a T {
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        (*self).render(f)
    }
}

impl Render for str {
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut last = 0;
        for (index, byte) in self.bytes().enumerate() {
            match byte {
                b'&' | b'<' | b'>' | b'"' => {
                    f.write_str(&self[last..index])?;
                    f.write_str(match byte {
                        b'&' => "&amp;",
                        b'<' => "&lt;",
                        b'>' => "&gt;",
                        _ => "&quot;",
                    })?;
                    last = index + 1;
                }
                _ => {}
            }
        }
        f.write_str(&self[last..])
    }
}

impl Render for String {
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.as_str().render(f)
    }
}

pub struct Raw<T: Display>(pub T);

impl<T: Display> Render for Raw<T> {
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! raw_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    self.fmt(f)
                }
            }
        )*
    };
}

raw_display! {
    bool
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    f32 f64
}

pub struct Doctype;

impl Render for Doctype {
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<!DOCTYPE html>")
    }
}