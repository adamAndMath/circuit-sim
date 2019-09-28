use super::hir::{ StateAst, Ast, Stmt, Func };
use pest_derive::Parser;
use pest::Parser;
type Pair<'i> = pest::iterators::Pair<'i, Rule>;

#[derive(Parser)]
#[grammar = "ast/cir.pest"]
struct CirParser;

pub fn parse<'a>(s: &'a str) -> Result<impl Iterator<Item = (String, Func)> + 'a, String> {
    CirParser::parse(Rule::file, s).map_err(|e|format!("{}", e)).map(|pairs|pairs.filter(|pair|pair.as_rule() != Rule::EOI).map(<(String, Func)>::parse))
}

trait Parse: Sized {
    fn parse(pair: Pair) -> Self;
}

impl<T: Parse> Parse for Box<T> {
    fn parse(pair: Pair) -> Self {
        Box::new(T::parse(pair))
    }
}

impl<T: Parse> Parse for Option<T> {
    fn parse(pair: Pair) -> Self {
        pair.into_inner().next().map(T::parse)
    }
}

impl<T: Parse> Parse for Vec<T> {
    fn parse(pair: Pair) -> Self {
        pair.into_inner().map(T::parse).collect()
    }
}

impl Parse for bool {
    fn parse(pair: Pair) -> Self {
        match pair.as_str() {
            "0" => false,
            "1" => true,
            p =>  unreachable!(p),
        }
    }
}

impl Parse for String {
    fn parse(pair: Pair) -> Self {
        debug_assert_eq!(pair.as_rule(), Rule::ident);
        pair.as_str().to_owned()
    }
}

impl Parse for (String, bool) {
    fn parse(pair: Pair) -> Self {
        assert_eq!(pair.as_rule(), Rule::state_def);
        let mut pairs = pair.into_inner();
        let name = pairs.next().map(String::parse).unwrap();
        let val = pairs.next().map(bool::parse).unwrap();
        (name, val)
    }
}

impl Parse for StateAst {
    fn parse(pair: Pair) -> Self {
        match pair.as_rule() {
            Rule::bool => StateAst::Const(bool::parse(pair)),
            Rule::ident => StateAst::Ident(String::parse(pair)),
            Rule::state_not => StateAst::Not(<Box<StateAst>>::parse(pair.into_inner().next().unwrap())),
            r => unreachable!("{:?}", r),
        }
    }
}

impl Parse for Ast {
    fn parse(pair: Pair) -> Self {
        match pair.as_rule() {
            Rule::bool => Ast::Source(bool::parse(pair)),
            Rule::ident => Ast::Wire(String::parse(pair)),
            Rule::ast_call => {
                let mut pairs = pair.into_inner();
                let ident = pairs.next().map(String::parse).unwrap();
                let state = pairs.next().map(<Option<Vec<StateAst>>>::parse).unwrap();
                let args = pairs.map(Ast::parse).collect();
                Ast::Call(ident, state, args)
            },
            r => unreachable!("{:?}", r),
        }
    }
}

impl Parse for Stmt {
    fn parse(pair: Pair) -> Self {
        match pair.as_rule() {
            Rule::stmt_float => {
                let pattern = <Vec<String>>::parse(pair);
                Stmt::Float(pattern)
            },
            Rule::stmt_let => {
                let mut pairs = pair.into_inner();
                let pattern = pairs.next().map(<Vec<Option<String>>>::parse).unwrap();
                let ast = pairs.next().map(Ast::parse).unwrap();
                Stmt::Let(pattern, ast)
            },
            Rule::stmt_set => {
                let mut pairs = pair.into_inner();
                let pattern = pairs.next().map(<Vec<Option<String>>>::parse).unwrap();
                let ast = pairs.next().map(Ast::parse).unwrap();
                Stmt::Set(pattern, ast)
            },
            Rule::bool | Rule::ast_call | Rule::ident => {
                Stmt::Call(Ast::parse(pair))
            },
            r => unreachable!("{:?}", r),
        }
    }
}

impl Parse for (String, Func) {
    fn parse(pair: Pair) -> Self {
        let mut pairs = pair.into_inner();
        let name = pairs.next().map(String::parse).unwrap();
        let func = Func {
            state: pairs.next().map(<Vec<(String, bool)>>::parse).unwrap(),
            input: pairs.next().map(<Vec<String>>::parse).unwrap(),
            output: pairs.next().map(<Vec<String>>::parse).unwrap(),
            stmts: pairs.map(Stmt::parse).collect(),
        };
        (name, func)
    }
}
