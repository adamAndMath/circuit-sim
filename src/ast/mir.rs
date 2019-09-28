use super::{ write_iter };
use circuit_sim::circuit::{ Circuit, Builder };
use circuit_sim::base::Component;
use std::convert::TryInto;
use std::fmt::{ self, Display, Formatter };

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
    pub state: Vec<StateAst>,
    pub input: Vec<usize>,
    pub output: Vec<usize>,
}

pub enum Func {
    Custom {
        state: usize,
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

pub struct FuncSign {
    pub id: usize,
    pub state: Vec<bool>,
    pub output: usize,
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
        write!(f, ") = {}[", self.func)?;
        write_iter(f, &self.state, ", ")?;
        write!(f, "](")?;
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
                writeln!(f, "[{}]({}) -> {} {{", state, input, output)?;
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
        let state: Vec<_> = self.state.iter().map(|s|s.eval(states)).collect();
        let input: Vec<_> = self.input.iter().map(|i|wires[*i]).collect();
        let output: Vec<_> = self.output.iter().map(|o|wires[*o]).collect();
        funcs[self.func].call(circuit, funcs, &state, &input, &output)
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
    fn call(&self, circuit: &mut Builder, funcs: &[Func], p_state: &[bool], p_input: &[usize], p_output: &[usize]) {
        match self {
            Func::Custom { local, stmts, .. } => {
                let state = p_state;
                let wires = p_input.iter()
                    .chain(p_output)
                    .copied()
                    .chain(std::iter::repeat_with(||circuit.new_slot()).take(*local))
                    .collect::<Vec<_>>();
                stmts.iter().for_each(|stmt|stmt.build(circuit, funcs, state, &wires))
            },
            Func::Source => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let []: [usize; 0] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Source(state), state);
            },
            Func::Buffer => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let [input]: [usize; 1] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Buffer(input), state);
            },
            Func::Inverter => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let [input]: [usize; 1] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Inverter(input), state);
            },
            Func::Or => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let [a, b]: [usize; 2] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Or(a, b), state);
            },
            Func::And => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let [a, b]: [usize; 2] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::And(a, b), state);
            },
            Func::Nor => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let [a, b]: [usize; 2] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Nor(a, b), state);
            },
            Func::Nand => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let [a, b]: [usize; 2] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Nand(a, b), state);
            },
            Func::Bus => {
                let [state]: [bool; 1] = p_state.try_into().unwrap();
                let []: [usize; 0] = p_input.try_into().unwrap();
                let [output]: [usize; 1] = p_output.try_into().unwrap();
                circuit.place_component(output, Component::Bus(vec![]), state);
            },
            Func::BusInput => {
                let []: [bool; 0] = p_state.try_into().unwrap();
                let [bus, a, b]: [usize; 3] = p_input.try_into().unwrap();
                let []: [usize; 0] = p_output.try_into().unwrap();
                circuit.add_bus_input(bus, a, b);
            },
        }
    }
    pub fn build_circuit(&self, funcs: &[Func], sign: &FuncSign) -> Circuit {
        let mut circuit = Circuit::builder();
        let input: Vec<_> = std::iter::repeat_with(||{
            let wire = circuit.new_slot();
            circuit.add_input(wire, false);
            wire
        }).take(self.input()).collect();
        let output: Vec<_> = std::iter::repeat_with(||{
            let wire = circuit.new_slot();
            circuit.add_output(wire);
            wire
        }).take(self.output()).collect::<Vec<_>>();
        self.call(&mut circuit, funcs, &sign.state, &input, &output);
        circuit.build()
    }
}
