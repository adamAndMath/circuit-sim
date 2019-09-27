use crate::env::Env;
use super::parser;
use circuit_sim::circuit::{ Circuit, Builder };
use circuit_sim::base::Component;
use std::convert::TryInto;
use std::fmt::{ self, Display, Formatter };

pub fn parse(s: &str) -> Result<(Vec<Func>, Env<(usize, usize)>), String> {
    let iter = parser::parse(s)?;
    let mut env = vec![
        ("source".to_owned(), (0, 1)),
        ("buffer".to_owned(), (1, 1)),
        ("not".to_owned(), (2, 1)),
        ("or".to_owned(), (3, 1)),
        ("and".to_owned(), (4, 1)),
        ("nor".to_owned(), (5, 1)),
        ("nand".to_owned(), (6, 1)),
        ("bus".to_owned(), (7, 1)),
        ("bus_input".to_owned(), (8, 0)),
    ].into_iter().collect();
    let mut funcs = vec![Func::Source, Func::Buffer, Func::Inverter, Func::Or, Func::And, Func::Nor, Func::Nand, Func::Bus, Func::BusInput];
    for (name, func) in iter {
        let output = func.output();
        let func = func.lower(&mut env);
        println!("{}{}", name, func);
        env.insert(name, (funcs.len(), output));
        funcs.push(func);
    }
    Ok((funcs, env))
}

enum StateRef {
    Const(bool),
    Ident(usize),
}

struct StateAst {
    negate: bool,
    state: StateRef,
}

impl Display for StateAst {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.negate {
            write!(f, "!")?;
        }
        match self.state {
            StateRef::Const(b) => write!(f, "{}", b),
            StateRef::Ident(i) => write!(f, "{}", i),
        }
    }
}

struct Stmt {
    func: usize,
    state: Option<Vec<StateAst>>,
    input: Vec<usize>,
    output: Vec<usize>,
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(")?;
        for i in &self.output {
            write!(f, "{}, ", i)?;
        }
        write!(f, ") = {}", self.func)?;
        if let Some(state) = &self.state {
            write!(f, "[")?;
            for state in state {
                write!(f, "{}, ", state)?;
            }
            write!(f, "]")?;
        }
        write!(f, "(")?;
        for i in &self.input {
            write!(f, "{}, ", i)?;
        }
        write!(f, ");")
    }
}

pub enum Func {
    Custom {
        state: Vec<bool>,
        input: usize,
        output: usize,
        local: usize,
        stmts: Vec<Stmt>,
    },
    Source,
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
            Func::Custom {
                state,
                input,
                output,
                local,
                stmts,
            } => {
                write!(f, "[")?;
                for b in state {
                    write!(f, "{}, ", b)?;
                }
                writeln!(f, "]({}) -> {} {{", input, output)?;
                writeln!(f, "  local {};", local)?;
                for stmt in stmts {
                    writeln!(f, "  {}", stmt)?;
                }
                write!(f, "}}")
            },
            Func::Source => write!(f, "source"),
            Func::Buffer => write!(f, "buffer"),
            Func::Inverter => write!(f, "not"),
            Func::Or => write!(f, "or"),
            Func::And => write!(f, "and"),
            Func::Nor => write!(f, "nor"),
            Func::Nand => write!(f, "nand"),
            Func::Bus => write!(f, "bus"),
            Func::BusInput => write!(f, "bus_input"),
        }
    }
}

impl super::StateAst {
    fn lower(self, states: &Env<usize>) -> StateAst {
        let mut negate = false;
        let mut ast = self;
        let state = loop {
            ast = match ast {
                super::StateAst::Const(b) => break StateRef::Const(b),
                super::StateAst::Ident(i) => break StateRef::Ident(states[&i]),
                super::StateAst::Not(inner) => {
                    negate = !negate;
                    *inner
                },
            }
        };
        StateAst { negate, state }
    }
}

impl super::Ast {
    fn lower_to(self, stmts: &mut Vec<Stmt>, wire_count: &mut usize, output: Vec<usize>, funcs: &Env<(usize, usize)>, states: &Env<usize>, wires: &Env<usize>) {
        match self {
            super::Ast::Source(b) => {
                let [output]: [usize; 1] = output.as_slice().try_into().unwrap();
                stmts.push(Stmt { func: 0, state: Some(vec![StateAst { negate: false, state: StateRef::Const(b) }]), input: vec![], output: vec![output] });
            },
            super::Ast::Wire(i) => panic!("Can't connect 2 wires"),
            super::Ast::Call(func, state, param) => {
                let (func, _) = funcs[&func];
                let state = state.map(|v|v.into_iter().map(|s|s.lower(states)).collect());
                let input = param.into_iter().flat_map(|ast|ast.lower(stmts, wire_count, funcs, states, wires)).collect();
                stmts.push(Stmt { func, state, input, output })
            },
        }
    }
    fn lower(self, stmts: &mut Vec<Stmt>, wire_count: &mut usize, funcs: &Env<(usize, usize)>, states: &Env<usize>, wires: &Env<usize>) -> Vec<usize> {
        match self {
            super::Ast::Source(b) => {
                let output = *wire_count;
                *wire_count += 1;
                stmts.push(Stmt { func: 0, state: Some(vec![StateAst { negate: false, state: StateRef::Const(b) }]), input: vec![], output: vec![output] });
                vec![output]
            },
            super::Ast::Wire(i) => vec![wires[&i]],
            super::Ast::Call(func, state, param) => {
                let (func, out) = funcs[&func];
                let state = state.map(|v|v.into_iter().map(|s|s.lower(states)).collect());
                let input = param.into_iter().flat_map(|ast|ast.lower(stmts, wire_count, funcs, states, wires)).collect();
                let output = (*wire_count..*wire_count+out).collect::<Vec<_>>();
                *wire_count += out;
                stmts.push(Stmt { func, state, input, output: output.clone() });
                output
            },
        }
    }
}

impl super::Stmt {
    fn lower(self, stmts: &mut Vec<Stmt>, wire_count: &mut usize, funcs: &Env<(usize, usize)>, states: &Env<usize>, wires: &mut Env<usize>) {
        match self {
            super::Stmt::Float(vec) => {
                let count = vec.len();
                wires.extend(vec.into_iter().zip(*wire_count..*wire_count+count));
                *wire_count += count;
            },
            super::Stmt::Let(vec, ast) => {
                let count = vec.len();
                let output = (*wire_count..*wire_count+count).collect::<Vec<_>>();
                wires.extend(vec.into_iter().zip(&output).flat_map(|(s, w)|s.map(|s|(s, *w))));
                *wire_count += count;
                ast.lower_to(stmts, wire_count, output, funcs, states, wires);
            },
            super::Stmt::Set(vec, ast) => {
                let output = vec.into_iter().map(|o|o.map_or_else(||{
                    let w = *wire_count;
                    *wire_count += 1;
                    w
                }, |o|wires[&o])).collect();
                ast.lower_to(stmts, wire_count, output, funcs, states, wires);
            },
            super::Stmt::Call(ast) => {
                ast.lower_to(stmts, wire_count, vec![], funcs, states, wires);
            },
        }
    }
}

impl super::Func {
    fn lower(self, funcs: &mut Env<(usize, usize)>) -> Func {
        match self {
            super::Func::Custom { state, input, output, stmts } => {
                let (state, states) = state.into_iter().enumerate().map(|(i, (s, v))|(v, (s, i))).unzip();
                let input_count = input.len();
                let output_count = output.len();
                let mut wire_count = input.len() + output.len();
                let mut wires = input.into_iter().chain(output).enumerate().map(|(i, s)|(s, i)).collect();
                let mut func_stmts = vec![];
                stmts.into_iter().for_each(|stmt|stmt.lower(&mut func_stmts, &mut wire_count, funcs, &states, &mut wires));
                Func::Custom {
                    state,
                    input: input_count,
                    output: output_count,
                    local: wire_count - input_count - output_count,
                    stmts: func_stmts,
                }
            },
            super::Func::Buffer => Func::Buffer,
            super::Func::Inverter => Func::Inverter,
            super::Func::Or => Func::Or,
            super::Func::And => Func::And,
            super::Func::Nor => Func::Nor,
            super::Func::Nand => Func::Nand,
            super::Func::Bus => Func::Bus,
            super::Func::BusInput => Func::BusInput,
        }
    }
}

impl StateAst {
    fn eval(&self, states: &[bool]) -> bool {
        self.negate ^ match self.state {
            StateRef::Const(b) => b,
            StateRef::Ident(i) => states[i],
        }
    }
}

impl Stmt {
    fn build(&self, circuit: &mut Builder, funcs: &[Func], states: &[bool], wires: &[usize]) {
        let state = self.state.as_ref().map(|v|v.iter().map(|s|s.eval(states)).collect());
        let input = self.input.iter().map(|i|wires[*i]).collect();
        let output = self.output.iter().map(|o|wires[*o]).collect();
        funcs[self.func].call(circuit, funcs, state, input, output)
    }
}

impl Func {
    pub fn input(&self) -> usize {
        match self {
            Func::Custom { input, .. } => *input,
            Func::Buffer |
            Func::Inverter => 1,
            Func::Or |
            Func::And |
            Func::Nor |
            Func::Nand => 2,
            Func::Source |
            Func::Bus => 0,
            Func::BusInput => 3,
        }
    }
    pub fn output(&self) -> usize {
        match self {
            Func::Custom { output, .. } => *output,
            Func::Source |
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
    fn call(&self, circuit: &mut Builder, funcs: &[Func], p_state: Option<Vec<bool>>, p_input: Vec<usize>, p_output: Vec<usize>) {
        match self {
            Func::Custom { state, local, stmts, .. } => {
                let state = p_state.as_ref().unwrap_or_else(||state).as_slice();
                let wires = p_input.into_iter()
                    .chain(p_output)
                    .chain(std::iter::repeat_with(||circuit.new_slot()).take(*local))
                    .collect::<Vec<_>>();
                stmts.iter().for_each(|stmt|stmt.build(circuit, funcs, state, &wires))
            },
            Func::Source => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let []: [usize; 0] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::Source(state), state);
            },
            Func::Buffer => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [input]: [usize; 1] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::Buffer(input), state);
            },
            Func::Inverter => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [input]: [usize; 1] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::Inverter(input), state);
            },
            Func::Or => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::Or(a, b), state);
            },
            Func::And => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::And(a, b), state);
            },
            Func::Nor => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::Nor(a, b), state);
            },
            Func::Nand => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let [a, b]: [usize; 2] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::Nand(a, b), state);
            },
            Func::Bus => {
                let [state] = p_state.map_or([false], |v|v.as_slice().try_into().unwrap());
                let []: [usize; 0] = p_input.as_slice().try_into().unwrap();
                let [output]: [usize; 1] = p_output.as_slice().try_into().unwrap();
                circuit.place_component(output, Component::Bus(vec![]), state);
            },
            Func::BusInput => {
                let [] = p_state.map_or([], |v|v.as_slice().try_into().unwrap());
                let [bus, a, b]: [usize; 3] = p_input.as_slice().try_into().unwrap();
                let []: [usize; 0] = p_output.as_slice().try_into().unwrap();
                circuit.add_bus_input(bus, a, b);
            },
        }
    }
    pub fn build_circuit(&self, funcs: &[Func]) -> Circuit {
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
        self.call(&mut circuit, funcs, None, input, output);
        circuit.build()
    }
}
