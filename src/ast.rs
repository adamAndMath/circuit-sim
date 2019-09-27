use super::env::Env;
use circuit_sim::circuit::{ Circuit, Builder };
use circuit_sim::base::Component;
use std::convert::TryInto;
use std::fmt::{ self, Display, Formatter };

mod parser;
pub mod mir;

pub fn parse(s: &str) -> Result<Env<Func>, String> {
    parser::parse(s).map(|iter|vec![
        ("buffer".to_owned(), Func::Buffer),
        ("not".to_owned(), Func::Inverter),
        ("or".to_owned(), Func::Or),
        ("and".to_owned(), Func::And),
        ("nor".to_owned(), Func::Nor),
        ("nand".to_owned(), Func::Nand),
        ("bus".to_owned(), Func::Bus),
        ("bus_input".to_owned(), Func::BusInput),
    ].into_iter().chain(iter)/*.inspect(|(ident, def)|println!("{}{}", ident, def))*/.collect())
}

enum StateAst {
    Const(bool),
    Ident(String),
    Not(Box<StateAst>),
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

enum Ast {
    Source(bool),
    Wire(String),
    Call(String, Option<Vec<StateAst>>, Vec<Ast>),
}

impl Display for Ast {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Ast::Source(b) => write!(f, "{}", b),
            Ast::Wire(i) => write!(f, "{}", i),
            Ast::Call(func, state, param) => {
                write!(f, "{}", func)?;
                if let Some(state) = state {
                    let mut sep = "[";
                    for state in state {
                        write!(f, "{}{}", sep, state)?;
                        sep = ", ";
                    }
                    write!(f, "]")?;
                }
                let mut sep = "(";
                for param in param {
                    write!(f, "{}{}", sep, param)?;
                    sep = ", ";
                }
                write!(f, ")")
            },
        }
    }
}

enum Stmt {
    Float(Vec<String>),
    Let(Vec<Option<String>>, Ast),
    Set(Vec<Option<String>>, Ast),
    Call(Ast),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Stmt::Float(pattern) => {
                let mut sep = "let (";
                for ident in pattern {
                    write!(f, "{}{}", sep, ident)?;
                    sep = ", ";
                }
                write!(f, ");")
            },
            Stmt::Let(pattern, ast) => {
                let mut sep = "let (";
                for ident in pattern {
                    match ident {
                        Some(ident) => write!(f, "{}{}", sep, ident)?,
                        None => write!(f, "{}_", sep)?,
                    }
                    sep = ", ";
                }
                write!(f, ") = {};", ast)
            },
            Stmt::Set(pattern, ast) => {
                let mut sep = "(";
                for ident in pattern {
                    match ident {
                        Some(ident) => write!(f, "{}{}", sep, ident)?,
                        None => write!(f, "{}_", sep)?,
                    }
                    sep = ", ";
                }
                write!(f, ") = {};", ast)
            },
            Stmt::Call(ast) => write!(f, "{};", ast),
        }
    }
}

pub enum Func {
    Custom {
        state: Vec<(String, bool)>,
        input: Vec<String>,
        output: Vec<String>,
        stmts: Vec<Stmt>,
    },
    Buffer,
    Inverter,
    Or,
    And,
    Nor,
    Nand,
    Bus,
    BusInput,
}

impl Display for Func {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Func::Custom { state, input, output, stmts } => {
                if !state.is_empty() {
                    write!(f, "[")?;
                    for (ident, val) in state {
                        write!(f, "{}={}, ", ident, val)?;
                    }
                    write!(f, "]")?;
                }
                write!(f, "(")?;
                for ident in input {
                    write!(f, "{}, ", ident)?;
                }
                write!(f, ") -> (")?;
                for ident in output {
                    write!(f, "{}, ", ident)?;
                }
                writeln!(f, ") {{")?;
                for stmt in stmts {
                    writeln!(f, "  {}", stmt)?;
                }
                write!(f, "}}")
            },
            Func::Buffer |
            Func::Inverter |
            Func::Or |
            Func::And |
            Func::Nor |
            Func::Nand |
            Func::Bus |
            Func::BusInput => Ok(()),
        }
    }
}

impl StateAst {
    fn eval(&self, states: &Env<bool>) -> bool {
        match self {
            StateAst::Const(b) => *b,
            StateAst::Ident(i) => states[i],
            StateAst::Not(ast) => !ast.eval(states),
        }
    }
}

impl Ast {
    fn build_to(&self, circuit: &mut Builder, funcs: &Env<Func>, states: &Env<bool>, wires: &Env<usize>, output: &[usize]) {
        match self {
            Ast::Source(b) => {
                let [w]: [_; 1] = output.try_into().unwrap();
                circuit.place_component(w, Component::Source(*b), *b);
            },
            Ast::Wire(_) => panic!("Can't connect 2 wires"),
            Ast::Call(func, state, param) => {
                let state = state.as_ref().map(|state|state.iter().map(|s|s.eval(states)).collect());
                let param = param.iter().flat_map(|p|p.build(circuit, funcs, states, wires)).collect();
                funcs[func].call(circuit, funcs, state, param, output);
            },
        }
    }
    fn build(&self, circuit: &mut Builder, funcs: &Env<Func>, states: &Env<bool>, wires: &Env<usize>) -> Vec<usize> {
        match self {
            Ast::Source(b) => {
                let wire = circuit.new_slot();
                circuit.place_component(wire, Component::Source(*b), *b);
                vec![wire]
            },
            Ast::Wire(wire) => vec![wires[wire]],
            Ast::Call(func, state, param) => {
                let state = state.as_ref().map(|state|state.iter().map(|s|s.eval(states)).collect());
                let param = param.iter().flat_map(|p|p.build(circuit, funcs, states, wires)).collect();
                let output = (0..funcs[func].output()).map(|_|circuit.new_slot()).collect::<Vec<_>>();
                funcs[func].call(circuit, funcs, state, param, &output);
                output
            },
        }
    }
}

impl Stmt {
    fn build(&self, circuit: &mut Builder, funcs: &Env<Func>, states: &Env<bool>, wires: &mut Env<usize>) {
        match self {
            Stmt::Float(ws) => ws.iter().cloned().for_each(|w| wires.insert(w, circuit.new_slot())),
            Stmt::Let(ws, ast) => {
                let output = ws.iter().cloned().map(|w| {
                    let id = circuit.new_slot();
                    if let Some(w) = w {
                        wires.insert(w, id);
                    }
                    id
                }).collect::<Vec<_>>();
                ast.build_to(circuit, funcs, states, wires, &output);
            },
            Stmt::Set(ws, ast) => {
                let output = ws.iter().map(|w|w.as_ref().map_or_else(||circuit.new_slot(), |w|wires[w])).collect::<Vec<_>>();
                ast.build_to(circuit, funcs, states, wires, &output);
            },
            Stmt::Call(ast) => {
                ast.build(circuit, funcs, states, wires);
            }
        }
    }
}

impl Func {
    pub fn input(&self) -> usize {
        match self {
            Func::Custom { input, .. } => input.len(),
            Func::Buffer |
            Func::Inverter => 1,
            Func::Or |
            Func::And |
            Func::Nor |
            Func::Nand => 2,
            Func::Bus => 0,
            Func::BusInput => 3,
        }
    }
    pub fn output(&self) -> usize {
        match self {
            Func::Custom { output, .. } => output.len(),
            Func::Buffer |
            Func::Inverter |
            Func::Or |
            Func::And |
            Func::Nor |
            Func::Nand |
            Func::Bus => 1,
            Func::BusInput => 0,
        }
    }
    fn call(&self, circuit: &mut Builder, funcs: &Env<Func>, p_state: Option<Vec<bool>>, p_input: Vec<usize>, p_output: &[usize]) {
        match self {
            Func::Custom { state, input, output, stmts } => {
                let states = p_state.map_or_else(||state.iter().cloned().collect(), |p_state|state.iter().map(|(s,_)|s.clone()).zip(p_state).collect());
                let mut wires = input.iter().cloned().zip(p_input).chain(output.iter().cloned().zip(p_output.iter().cloned())).collect();
                stmts.iter().for_each(|stmt|stmt.build(circuit, funcs, &states, &mut wires));
            },
            Func::Buffer => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [input]: [usize; 1] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Buffer(input), state);
            },
            Func::Inverter => {
                let [state] = p_state.map_or([true], |v|v.as_slice().try_into().unwrap());
                let [input]: [usize; 1] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Inverter(input), state);
            },
            Func::Or => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Or(a, b), state);
            },
            Func::And => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::And(a, b), state);
            },
            Func::Nor => {
                let [state] = p_state.map_or([true], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Nor(a, b), state);
            },
            Func::Nand => {
                let [state] = p_state.map_or([true], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Nand(a, b), state);
            },
            Func::Bus => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let []: [usize; 0] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Bus(vec![]), state);
            },
            Func::BusInput => {
                let [] = p_state.map_or([], |v|v.as_slice().try_into().unwrap());
                let [bus, a, b]: [usize; 3] = p_input.as_slice().try_into().unwrap();
                let []: [usize; 0] = p_output.try_into().unwrap();
                circuit.add_bus_input(bus, a, b);
            },
        }
    }
    pub fn build_circuit(&self, funcs: &Env<Func>) -> Circuit {
        let mut circuit = Circuit::builder();
        let input = std::iter::repeat_with(||{
            let wire = circuit.new_slot();
            circuit.add_input(wire, false);
            wire
        }).take(self.input()).collect();
        let output = std::iter::repeat_with(||{
            let wire = circuit.new_slot();
            circuit.add_output(wire);
            wire
        }).take(self.output()).collect::<Vec<_>>();
        self.call(&mut circuit, funcs, None, input, &output);
        circuit.build()
    }
}
