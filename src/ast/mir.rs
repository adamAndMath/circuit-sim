use crate::env::Env;
use super::{ parser, write_iter };
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
        let output = func.output.len();
        let func = func.lower(&env);
        println!("{}{}", name, func);
        env.insert(name, (funcs.len(), output));
        funcs.push(func);
    }
    Ok((funcs, env))
}

pub enum StateRef {
    Const(bool),
    Ident(usize),
}

pub struct StateAst {
    pub negate: bool,
    pub state: StateRef,
}

pub struct Stmt {
    pub func: usize,
    pub state: Option<Vec<StateAst>>,
    pub input: Vec<usize>,
    pub output: Vec<usize>,
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

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "(")?;
        write_iter(f, &self.output, ", ")?;
        write!(f, ") = {}", self.func)?;
        if let Some(state) = &self.state {
            write!(f, "[")?;
            write_iter(f, state, ", ")?;
            write!(f, "]")?;
        }
        write!(f, "(")?;
        write_iter(f, &self.input, ", ")?;
        write!(f, ");")
    }
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
                write_iter(f, state, ", ")?;
                writeln!(f, "]({}) -> {} {{", input, output)?;
                writeln!(f, "  local {};", local)?;
                for stmt in stmts {
                    writeln!(f, "  {}", stmt)?;
                }
                write!(f, "}}")
            },
            _ => Ok(()),
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
