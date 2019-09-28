use std::fmt::{ self, Display, Formatter };
use crate::env::Env;

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

pub fn parse(s: &str) -> Result<(Vec<mir::Func>, Env<mir::FuncSign>), String> {
    let iter = parser::parse(s)?;
    let mut env = vec![
        ("source".to_owned(), mir::FuncSign { id: 0, state: vec![false], output: 1 }),
        ("buffer".to_owned(), mir::FuncSign { id: 1, state: vec![false], output: 1 }),
        ("not".to_owned(), mir::FuncSign { id: 2, state: vec![true], output: 1 }),
        ("or".to_owned(), mir::FuncSign { id: 3, state: vec![false], output: 1 }),
        ("and".to_owned(), mir::FuncSign { id: 4, state: vec![false], output: 1 }),
        ("nor".to_owned(), mir::FuncSign { id: 5, state: vec![true], output: 1 }),
        ("nand".to_owned(), mir::FuncSign { id: 6, state: vec![true], output: 1 }),
        ("bus".to_owned(), mir::FuncSign { id: 7, state: vec![false], output: 1 }),
        ("bus_input".to_owned(), mir::FuncSign { id: 8, state: vec![], output: 0 }),
    ].into_iter().collect();
    let mut funcs = vec![mir::Func::Source, mir::Func::Buffer, mir::Func::Inverter, mir::Func::Or, mir::Func::And, mir::Func::Nor, mir::Func::Nand, mir::Func::Bus, mir::Func::BusInput];
    for (name, func) in iter {
        let (sign, func) = func.lower(&env, funcs.len());
        println!("{}{}", name, func);
        env.insert(name, sign);
        funcs.push(func);
    }
    Ok((funcs, env))
}
