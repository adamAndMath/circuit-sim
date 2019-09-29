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

fn single_state(func: &str, default: bool, state: Option<Vec<StateAst>>, states: &Env<usize>) -> mir::StateAst {
    match state {
        None => default.into(),
        Some(v) => {
            if v.len() != 1 {
                panic!("{} takes 1 state, but recieved {}", func, v.len())
            } else {
                v.into_iter().next().unwrap().lower(states)
            }
        },
    }
}

fn expect_input(func: &str, expected: usize, input: usize) {
    if input != expected {
        panic!("{} takes {} input, but recieved {}", func, expected, input)
    }
}

fn expect_io(func: &str, exp_in: usize, exp_out: usize, input: usize, output: usize) {
    if input != exp_in {
        panic!("{} takes {} input, but recieved {}", func, exp_in, input)
    }
    if output != exp_out {
        panic!("{} gives {} output, but expected {}", func, exp_out, output)
    }
}

impl Ast {
    fn lower_to(self, stmts: &mut Vec<mir::Stmt>, wire_count: &mut usize, output: Vec<usize>, funcs: &Env<mir::FuncSign>, states: &Env<usize>, wires: &Env<usize>) {
        match self {
            Ast::Source(b) => {
                let [output]: [usize; 1] = output.as_slice().try_into().unwrap();
                stmts.push(mir::Stmt::Source(b.into(), output));
            },
            Ast::Wire(_) => panic!("Can't connect 2 wires"),
            Ast::Call(func, state, param) => {
                let input: Vec<_> = param.into_iter().flat_map(|ast|ast.lower(stmts, wire_count, funcs, states, wires)).collect();
                match func.as_str() {
                    "buffer" => {
                        let state = single_state("buffer", false, state, states);
                        expect_io("buffer", 1, 1, input.len(), output.len());
                        stmts.push(mir::Stmt::Buffer(state, input[0], output[0]))
                    },
                    "not" => {
                        let state = single_state("not", true, state, states);
                        expect_io("not", 1, 1, input.len(), output.len());
                        stmts.push(mir::Stmt::Inverter(state, input[0], output[0]))
                    },
                    "or" => {
                        let state = single_state("or", false, state, states);
                        expect_io("or", 2, 1, input.len(), output.len());
                        stmts.push(mir::Stmt::Or(state, input[0], input[1], output[0]))
                    },
                    "and" => {
                        let state = single_state("and", false, state, states);
                        expect_io("and", 2, 1, input.len(), output.len());
                        stmts.push(mir::Stmt::And(state, input[0], input[1], output[0]))
                    },
                    "nor" => {
                        let state = single_state("nor", true, state, states);
                        expect_io("nor", 2, 1, input.len(), output.len());
                        stmts.push(mir::Stmt::Nor(state, input[0], input[1], output[0]))
                    },
                    "nand" => {
                        let state = single_state("nand", true, state, states);
                        expect_io("nand", 2, 1, input.len(), output.len());
                        stmts.push(mir::Stmt::Nand(state, input[0], input[1], output[0]))
                    },
                    "bus" => {
                        let state = single_state("bus", false, state, states);
                        expect_io("bus", 0, 1, input.len(), output.len());
                        stmts.push(mir::Stmt::Bus(state, output[0]))
                    },
                    "bus_input" => {
                        if let Some(state) = state {
                            if !state.is_empty() {
                                panic!("bus_input takes 0 state, but recieved {}", state.len())
                            }
                        }
                        expect_io("bus_input", 3, 0, input.len(), output.len());
                        stmts.push(mir::Stmt::BusInput(input[0], input[1], input[2]))
                    },
                    func => {
                        let func = &funcs[func];
                        let state: Vec<_> = match state {
                            None => func.state.iter().map(|b|mir::StateAst { negate: false, state: mir::StateRef::Const(*b) }).collect(), 
                            Some(v) => v.into_iter().map(|s|s.lower(states)).collect(),
                        };
                        expect_io("bus_input", func.input, func.output, input.len(), output.len());
                        let mut wires = input;
                        wires.extend(output);
                        stmts.push(mir::Stmt::Call { func: func.id, state, wires })
                    },
                }
            },
        }
    }
    fn lower(self, stmts: &mut Vec<mir::Stmt>, wire_count: &mut usize, funcs: &Env<mir::FuncSign>, states: &Env<usize>, wires: &Env<usize>) -> Vec<usize> {
        match self {
            Ast::Source(b) => {
                let output = *wire_count;
                *wire_count += 1;
                stmts.push(mir::Stmt::Source(b.into(), output));
                vec![output]
            },
            Ast::Wire(i) => vec![wires[&i]],
            Ast::Call(func, state, param) => {
                let input: Vec<_> = param.into_iter().flat_map(|ast|ast.lower(stmts, wire_count, funcs, states, wires)).collect();
                match func.as_str() {
                    "source" => {
                        let state = single_state("source", false, state, states);
                        expect_input("source", 1, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::Buffer(state, input[0], output));
                        vec![output]
                    },
                    "buffer" => {
                        let state = single_state("buffer", false, state, states);
                        expect_input("buffer", 1, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::Buffer(state, input[0], output));
                        vec![output]
                    },
                    "not" => {
                        let state = single_state("not", true, state, states);
                        expect_input("not", 1, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::Inverter(state, input[0], output));
                        vec![output]
                    },
                    "or" => {
                        let state = single_state("or", false, state, states);
                        expect_input("or", 2, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::Or(state, input[0], input[1], output));
                        vec![output]
                    },
                    "and" => {
                        let state = single_state("and", false, state, states);
                        expect_input("and", 2, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::And(state, input[0], input[1], output));
                        vec![output]
                    },
                    "nor" => {
                        let state = single_state("nor", true, state, states);
                        expect_input("nor", 2, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::Nor(state, input[0], input[1], output));
                        vec![output]
                    },
                    "nand" => {
                        let state = single_state("nand", true, state, states);
                        expect_input("nand", 2, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::Nand(state, input[0], input[1], output));
                        vec![output]
                    },
                    "bus" => {
                        let state = single_state("bus", false, state, states);
                        expect_input("bus", 0, input.len());
                        let output = *wire_count;
                        *wire_count += 1;
                        stmts.push(mir::Stmt::Bus(state, output));
                        vec![output]
                    },
                    "bus_input" => {
                        if let Some(state) = state {
                            if !state.is_empty() {
                                panic!("bus_input takes 0 state, but recieved {}", state.len())
                            }
                        }
                        expect_input("bus_input", 0, input.len());
                        stmts.push(mir::Stmt::BusInput(input[0], input[1], input[2]));
                        vec![]
                    },
                    func => {
                        let func = &funcs[func];
                        let state: Vec<_> = match state {
                            None => func.state.iter().map(|b|mir::StateAst { negate: false, state: mir::StateRef::Const(*b) }).collect(), 
                            Some(v) => v.into_iter().map(|s|s.lower(states)).collect(),
                        };
                        let mut wires = input;
                        let output = (*wire_count..*wire_count+func.output).collect::<Vec<_>>();
                        wires.extend(*wire_count..*wire_count+func.output);
                        *wire_count += func.output;
                        stmts.push(mir::Stmt::Call { func: func.id, state, wires });
                        output
                    },
                }
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
        let (sign_state, states) = self.state.into_iter().enumerate().map(|(i, (s, v))|(v, (s, i))).unzip();
        let input_count = self.input.len();
        let output_count = self.output.len();
        let mut wire_count = self.input.len() + self.output.len();
        let mut wires = self.input.into_iter().chain(self.output).enumerate().map(|(i, s)|(s, i)).collect();
        let mut stmts = vec![];
        self.stmts.into_iter().for_each(|stmt|stmt.lower(&mut stmts, &mut wire_count, funcs, &states, &mut wires));
        (
            mir::FuncSign {
                id,
                state: sign_state,
                input: input_count,
                output: output_count,
            },
            mir::Func {
                local: wire_count - input_count - output_count,
                stmts,
            }
        )
    }
}
