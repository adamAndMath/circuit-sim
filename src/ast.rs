use std::fmt::{ self, Display, Formatter };

pub mod hir;
pub mod mir;
mod parser;

fn write_iter<I: IntoIterator>(f: &mut Formatter, iter: I, sep: &str) -> fmt::Result where I::Item: Display {
    let mut iter = iter.into_iter();
    if let Some(first) = iter.next() {
        first.fmt(f)?;
        for item in iter {
            f.write_str(sep)?;
            item.fmt(f)?;
        }
    }
    Ok(())
}

fn write_iter_with<I: IntoIterator, F: FnMut(I::Item, &mut Formatter) -> fmt::Result>(f: &mut Formatter, iter: I, mut elm: F, sep: &str) -> fmt::Result {
    let mut iter = iter.into_iter();
    if let Some(first) = iter.next() {
        elm(first, f)?;
        for item in iter {
            f.write_str(sep)?;
            elm(item, f)?;
        }
    }
    Ok(())
}
