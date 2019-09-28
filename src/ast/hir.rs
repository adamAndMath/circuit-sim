use super::{ write_iter, write_iter_with };
use std::fmt::{ self, Display, Formatter };

mod to_mir;

pub enum StateAst {
    Const(bool),
    Ident(String),
    Not(Box<StateAst>),
}

pub enum Ast {
    Source(bool),
    Wire(String),
    Call(String, Option<Vec<StateAst>>, Vec<Ast>),
}

pub enum Stmt {
    Float(Vec<String>),
    Let(Vec<Option<String>>, Ast),
    Set(Vec<Option<String>>, Ast),
    Call(Ast),
}

pub struct Func {
    pub state: Vec<(String, bool)>,
    pub input: Vec<String>,
    pub output: Vec<String>,
    pub stmts: Vec<Stmt>,
}

impl Display for StateAst {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            StateAst::Const(b) => write!(f, "{}", b),
            StateAst::Ident(i) => write!(f, "{}", i),
            StateAst::Not(ast) => write!(f, "!{}", ast),
        }
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ast::Source(b) => write!(f, "{}", b),
            Ast::Wire(i) => write!(f, "{}", i),
            Ast::Call(func, state, param) => {
                write!(f, "{}", func)?;
                if let Some(state) = state {
                    write!(f, "[")?;
                    write_iter(f, state, ", ")?;
                    write!(f, "]")?;
                }
                write!(f, "(")?;
                write_iter(f, param, ", ")?;
                write!(f, ")")
            },
        }
    }
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Stmt::Float(pattern) => {
                write!(f, "let (")?;
                write_iter(f, pattern, ", ")?;
                write!(f, ");")
            },
            Stmt::Let(pattern, ast) => {
                write!(f, "let (")?;
                write_iter_with(f, pattern, |ident, f|match ident {
                    Some(ident) => write!(f, "{}", ident),
                    None => write!(f, "_"),
                }, ", ")?;
                write!(f, ") = {};", ast)
            },
            Stmt::Set(pattern, ast) => {
                write!(f, "(")?;
                write_iter_with(f, pattern, |ident, f|match ident {
                    Some(ident) => write!(f, "{}", ident),
                    None => write!(f, "_"),
                }, ", ")?;
                write!(f, ") = {};", ast)
            },
            Stmt::Call(ast) => write!(f, "{};", ast),
        }
    }
}

impl Display for Func {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if !self.state.is_empty() {
            write!(f, "[")?;
            write_iter_with(f, &self.state, |(ident, val), f|write!(f, "{}={}", ident, val), ", ")?;
            write!(f, "]")?;
        }
        write!(f, "(")?;
        write_iter(f, &self.input, ", ")?;
        write!(f, ") -> (")?;
        write_iter(f, &self.output, ", ")?;
        writeln!(f, ") {{")?;
        for stmt in &self.stmts {
            writeln!(f, "  {}", stmt)?;
        }
        write!(f, "}}")
    }
}
