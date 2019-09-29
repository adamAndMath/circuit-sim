use super::{ write_iter };
use circuit_sim::circuit::{ Circuit, Builder };
use circuit_sim::base::Component;
use std::fmt::{ self, Display, Formatter };

pub enum StateRef {
    Const(bool),
    Ident(usize),
}

pub struct StateAst {
    pub negate: bool,
    pub state: StateRef,
}

impl From<bool> for StateAst {
    fn from(b: bool) -> Self {
        StateAst {
            negate: false,
            state: StateRef::Const(b),
        }
    }
}

pub enum Stmt {
    Call {
        func: usize,
        state: Vec<StateAst>,
        wires: Vec<usize>,
    },
    Source(StateAst, usize),
    Buffer(StateAst, usize, usize),
    Inverter(StateAst, usize, usize),
    Or(StateAst, usize, usize, usize),
    And(StateAst, usize, usize, usize),
    Nor(StateAst, usize, usize, usize),
    Nand(StateAst, usize, usize, usize),
    Bus(StateAst, usize),
    BusInput(usize, usize, usize),
}

pub struct FuncSign {
    pub id: usize,
    pub state: Vec<bool>,
    pub input: usize,
    pub output: usize,
}

pub struct Func {
    pub local: usize,
    pub stmts: Vec<Stmt>,
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
        match self {
            Stmt::Call { func, state, wires } => {
                write!(f, "{}[", func)?;
                write_iter(f, state, ", ")?;
                write!(f, "](")?;
                write_iter(f, wires, ", ")?;
                write!(f, ");")
            },
            Stmt::Source(state, input) => write!(f, "source[{}]({});", state, input),
            Stmt::Buffer(state, input, output) => write!(f, "{} = buffer[{}]({});", output, state, input),
            Stmt::Inverter(state, input, output) => write!(f, "{} = not[{}]({});", output, state, input),
            Stmt::Or(state, a, b, o) => write!(f, "{} = or[{}]({}, {});", o, state, a, b),
            Stmt::And(state, a, b, o) => write!(f, "{} = and[{}]({}, {});", o, state, a, b),
            Stmt::Nor(state, a, b, o) => write!(f, "{} = nor[{}]({}, {});", o, state, a, b),
            Stmt::Nand(state, a, b, o) => write!(f, "{} = nand[{}]({}, {});", o, state, a, b),
            Stmt::Bus(state, output) => write!(f, "{} = bus[{}]();", output, state),
            Stmt::BusInput(bus, high, low) => write!(f, "bus_input({}, {}, {})", bus, high, low),
        }
    }
}

impl Display for FuncSign {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[")?;
        write_iter(f, &self.state, ", ")?;
        write!(f, "]({}) -> ({})", self.input, self.output)
    }
}

impl Display for Func {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(f, "{{")?;
        writeln!(f, "  local {};", self.local)?;
        for stmt in &self.stmts {
            writeln!(f, "  {}", stmt)?;
        }
        write!(f, "}}")
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
    fn build(&self, circuit: &mut Builder, funcs: &[Func], states: &[bool], p_wires: &[usize]) {
        match self {
            Stmt::Call { func, state, wires } => {
                let state: Vec<_> = state.iter().map(|s|s.eval(states)).collect();
                let wires: Vec<_> = wires.iter().map(|i|p_wires[*i]).collect();
                funcs[*func].call(circuit, funcs, &state, &wires)
            },
            Stmt::Source(state, output) => {
                let state = state.eval(states);
                circuit.place_component(p_wires[*output], Component::Source(state), state);
            },
            Stmt::Buffer(state, input, output) => {
                circuit.place_component(p_wires[*output], Component::Buffer(p_wires[*input]), state.eval(states));
            },
            Stmt::Inverter(state, input, output) => {
                circuit.place_component(p_wires[*output], Component::Inverter(p_wires[*input]), state.eval(states));
            },
            Stmt::Or(state, a, b, o) => {
                circuit.place_component(p_wires[*o], Component::Or(p_wires[*a], p_wires[*b]), state.eval(states));
            },
            Stmt::And(state, a, b, o) => {
                circuit.place_component(p_wires[*o], Component::And(p_wires[*a], p_wires[*b]), state.eval(states));
            },
            Stmt::Nor(state, a, b, o) => {
                circuit.place_component(p_wires[*o], Component::Nor(p_wires[*a], p_wires[*b]), state.eval(states));
            },
            Stmt::Nand(state, a, b, o) => {
                circuit.place_component(p_wires[*o], Component::Nand(p_wires[*a], p_wires[*b]), state.eval(states));
            },
            Stmt::Bus(state, output) => {
                circuit.place_component(p_wires[*output], Component::Bus(vec![]), state.eval(states));
            },
            Stmt::BusInput(bus, a, b) => {
                circuit.add_bus_input(p_wires[*bus], p_wires[*a], p_wires[*b]);
            },
        }
    }
}

impl Func {
    fn call(&self, circuit: &mut Builder, funcs: &[Func], p_state: &[bool], p_wires: &[usize]) {
        let state = p_state;
        let wires = p_wires.iter().copied()
            .chain(std::iter::repeat_with(||circuit.new_slot()).take(self.local))
            .collect::<Vec<_>>();
        self.stmts.iter().for_each(|stmt|stmt.build(circuit, funcs, state, &wires))
    }
    pub fn build_circuit(&self, funcs: &[Func], sign: &FuncSign) -> Circuit {
        let mut circuit = Circuit::builder();
        let mut wires = Vec::with_capacity(sign.input + sign.output);
        wires.extend(std::iter::repeat_with(||{
            let wire = circuit.new_slot();
            circuit.add_input(wire, false);
            wire
        }).take(sign.input));
        wires.extend(std::iter::repeat_with(||{
            let wire = circuit.new_slot();
            circuit.add_output(wire);
            wire
        }).take(sign.output));
        self.call(&mut circuit, funcs, &sign.state, &wires);
        circuit.build()
    }
}
