use super::{ StateAst, Ast, Stmt, Func };
use crate::ast::mir;
use crate::env::Env;
use std::convert::TryInto;

impl StateAst {
    fn lower(self, states: &Env<usize>) -> mir::StateAst {
        let mut negate = false;
        let mut ast = self;
        let state = loop {
            ast = match ast {
                StateAst::Const(b) => break mir::StateRef::Const(b),
                StateAst::Ident(i) => break mir::StateRef::Ident(states[&i]),
                StateAst::Not(inner) => {
                    negate = !negate;
                    *inner
                },
            }
        };
        mir::StateAst { negate, state }
    }
}

impl Ast {
    fn lower_to(self, stmts: &mut Vec<mir::Stmt>, wire_count: &mut usize, output: Vec<usize>, funcs: &Env<mir::FuncSign>, states: &Env<usize>, wires: &Env<usize>) {
        match self {
            Ast::Source(b) => {
                let [output]: [usize; 1] = output.as_slice().try_into().unwrap();
                stmts.push(mir::Stmt { func: 0, state: vec![mir::StateAst { negate: false, state: mir::StateRef::Const(b) }], input: vec![], output: vec![output] });
            },
            Ast::Wire(_) => panic!("Can't connect 2 wires"),
            Ast::Call(func, state, param) => {
                let func = &funcs[&func];
                let state: Vec<_> = match state {
                    None => func.state.iter().map(|b|mir::StateAst { negate: false, state: mir::StateRef::Const(*b) }).collect(), 
                    Some(v) => v.into_iter().map(|s|s.lower(states)).collect(),
                };
                let input = param.into_iter().flat_map(|ast|ast.lower(stmts, wire_count, funcs, states, wires)).collect();
                stmts.push(mir::Stmt { func: func.id, state, input, output })
            },
        }
    }
    fn lower(self, stmts: &mut Vec<mir::Stmt>, wire_count: &mut usize, funcs: &Env<mir::FuncSign>, states: &Env<usize>, wires: &Env<usize>) -> Vec<usize> {
        match self {
            Ast::Source(b) => {
                let output = *wire_count;
                *wire_count += 1;
                stmts.push(mir::Stmt { func: 0, state: vec![mir::StateAst { negate: false, state: mir::StateRef::Const(b) }], input: vec![], output: vec![output] });
                vec![output]
            },
            Ast::Wire(i) => vec![wires[&i]],
            Ast::Call(func, state, param) => {
                let func = &funcs[&func];
                let state: Vec<_> = match state {
                    None => func.state.iter().map(|b|mir::StateAst { negate: false, state: mir::StateRef::Const(*b) }).collect(), 
                    Some(v) => v.into_iter().map(|s|s.lower(states)).collect(),
                };
                let input = param.into_iter().flat_map(|ast|ast.lower(stmts, wire_count, funcs, states, wires)).collect();
                let output = (*wire_count..*wire_count+func.output).collect::<Vec<_>>();
                *wire_count += func.output;
                stmts.push(mir::Stmt { func: func.id, state, input, output: output.clone() });
                output
            },
        }
    }
}

impl Stmt {
    fn lower(self, stmts: &mut Vec<mir::Stmt>, wire_count: &mut usize, funcs: &Env<mir::FuncSign>, states: &Env<usize>, wires: &mut Env<usize>) {
        match self {
            Stmt::Float(vec) => {
                let count = vec.len();
                wires.extend(vec.into_iter().zip(*wire_count..*wire_count+count));
                *wire_count += count;
            },
            Stmt::Let(vec, ast) => {
                let count = vec.len();
                let output = (*wire_count..*wire_count+count).collect::<Vec<_>>();
                wires.extend(vec.into_iter().zip(&output).flat_map(|(s, w)|s.map(|s|(s, *w))));
                *wire_count += count;
                ast.lower_to(stmts, wire_count, output, funcs, states, wires);
            },
            Stmt::Set(vec, ast) => {
                let output = vec.into_iter().map(|o|o.map_or_else(||{
                    let w = *wire_count;
                    *wire_count += 1;
                    w
                }, |o|wires[&o])).collect();
                ast.lower_to(stmts, wire_count, output, funcs, states, wires);
            },
            Stmt::Call(ast) => {
                ast.lower_to(stmts, wire_count, vec![], funcs, states, wires);
            },
        }
    }
}

impl Func {
    pub fn lower(self, funcs: &Env<mir::FuncSign>, id: usize) -> (mir::FuncSign, mir::Func) {
        let state = self.state.len();
        let (sign_state, states) = self.state.into_iter().enumerate().map(|(i, (s, v))|(v, (s, i))).unzip();
        let input_count = self.input.len();
        let output_count = self.output.len();
        let mut wire_count = self.input.len() + self.output.len();
        let mut wires = self.input.into_iter().chain(self.output).enumerate().map(|(i, s)|(s, i)).collect();
        let mut func_stmts = vec![];
        self.stmts.into_iter().for_each(|stmt|stmt.lower(&mut func_stmts, &mut wire_count, funcs, &states, &mut wires));
        (
            mir::FuncSign {
                id,
                state: sign_state,
                output: output_count,
            },
            mir::Func::Custom {
                state,
                input: input_count,
                output: output_count,
                local: wire_count - input_count - output_count,
                stmts: func_stmts,
            }
        )
    }
}
