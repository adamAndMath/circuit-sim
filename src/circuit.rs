use crate::base::{Component, WholeNew, WholeNewState, Wire};

#[derive(Default)]
pub struct Circuit {
  whole_new: WholeNew,
  inputs: Vec<usize>,
  outputs: Vec<usize>,
}

impl Circuit {
  pub fn new_state(&mut self) -> WholeNewState {
    self.whole_new.new_state()
  }
  pub fn update(&mut self, state: &mut WholeNewState) {
    self.whole_new.update(state);
  }
  pub fn set_input(&mut self, inputs: Vec<bool>) -> Result<(), String> {
    if self.inputs.len() != inputs.len() {
      return Err(format!("Expected {} inputs, but recieved {}", self.inputs.len(), inputs.len()))
    }
    for (input, arg) in self.inputs.iter().zip(inputs) {
      match &mut self.whole_new.components[*input].0 {
        Component::Source(v) => *v = arg,
        _ => return Err(format!("{} is not a source", input)),
      }
    }
    Ok(())
  }
  pub fn print_output(&self, state: &WholeNewState) {
    self.outputs.iter().copied().map(|i|state.wires[i]).for_each(|b|if b { print!("1") } else { print!("0") });
    println!();
  }
  pub fn add_wire(&mut self) -> usize {
    let id = self.whole_new.wires.len();
    self.whole_new.wires.push(Wire::default());
    id
  }
}
/* impl Circuit {
  pub fn tri_state(&mut self, i: usize, e: usize, o: usize) {
    let inv = self.add_wire();
    let pos = self.add_wire();
    let neg = self.add_wire();
    self.inverter(i, inv);
    self.and(e, i, pos);
    self.and(e, inv, neg);
    self.snowflake(pos, neg, o);
  }
  pub fn xor(&mut self, i0: usize, i1: usize, o: usize) {
    let nand = self.add_wire();
    let or = self.add_wire();
    self.nand(i0, i1, nand);
    self.or(i0, i1, or);
    self.and(nand, or, o);
  }
  pub fn xnor(&mut self, i0: usize, i1: usize, o: usize) {
    let and = self.add_wire();
    let nor = self.add_wire();
    self.and(i0, i1, and);
    self.nor(i0, i1, nor);
    self.or(and, nor, o);
  }
  pub fn flip_flop(&mut self, r: usize, s: usize, q: usize, qn: usize) {
    self.nor(r, qn, q);
    self.nor(s, q, qn);
  }
  pub fn half_adder(&mut self, i0: usize, i1: usize, s: usize, c: usize) {
    self.xor(i0, i1, s);
    self.and(i0, i1, c);
  }
  pub fn full_adder(&mut self, i0: usize, i1: usize, c_in: usize, s: usize, c_out: usize) {
    let sum = self.add_wire();
    let c0 = self.add_wire();
    let c1 = self.add_wire();
    self.half_adder(i0, i1, sum, c0);
    self.xor(sum, c_in, s);
    self.and(sum, c_in, c1);
    self.and(c0, c1, c_out);
  }
} */
#[must_use]
pub fn component(component: Component, default: Option<bool>) -> impl FnOnce(&mut Circuit, usize) {
  move |circuit, o| {
    let id = circuit.whole_new.components.len();
    circuit.whole_new.components.push((component, default));
    circuit.whole_new.wires[o].inputs.push(id);
  }
}
#[must_use]
pub fn input(default: bool) -> impl FnOnce(&mut Circuit, usize) {
  move |circuit, o: usize| {
    let id = circuit.whole_new.components.len();
    circuit.whole_new.components.push((Component::Source(default), Some(default)));
    circuit.whole_new.wires[o].inputs.push(id);
    circuit.inputs.push(id);
  }
}
#[must_use]
pub fn output(i: usize) -> impl FnOnce(&mut Circuit) {
  move |circuit| { circuit.outputs.push(i) }
}
define! {
  pub fn buffer(i) -> (o) {
    o = component(Component::Buffer(i), Some(false));
  }
  pub fn inverter(i) -> (o) {
    o = component(Component::Inverter(i), Some(true));
  }
  pub fn or(i0, i1) -> (o) {
    o = component(Component::Or(i0, i1), Some(false));
  }
  pub fn and(i0, i1) -> (o) {
    o = component(Component::And(i0, i1), Some(false));
  }
  pub fn nor(i0, i1) -> (o) {
    o = component(Component::Nor(i0, i1), Some(true));
  }
  pub fn nand(i0, i1) -> (o) {
    o = component(Component::Nand(i0, i1), Some(true));
  }
  fn snowflake(i0, i1) -> (o) {
    o = component(Component::Snowflake(i0, i1), None);
  }
}

pub fn clock(n: usize) -> impl FnOnce(&mut Circuit, usize) {
  move |circuit, o| {
    let mut wire = o;
    for _ in 1..n {
      let f = buffer(wire);
      wire = circuit.add_wire();
      f(circuit, wire);
    }
    inverter(wire)(circuit, o);
  }
}

define! {
  //0
  pub fn xor(i0, i1) -> (o) {
    let nand = nand(i0, i1);
    let or = or(i0, i1);
    o = and(nand, or);
  }
  pub fn xnor(i0, i1) -> (o) {
    let and = and(i0, i1);
    let nor = nor(i0, i1);
    o = or(and, nor);
  }
  pub fn bin_deco(i, e) -> (dir, inv) {
    let not = inverter(i);
    dir = and(e, i);
    inv = and(e, not);
  }
  pub fn sr_latch(s, r) -> (q, qn) {
    q = component(Component::Nor(r, qn), Some(false));
    qn = nor(s, q);
  }
  pub fn rising_edge(i) -> (o) {
    let buf = buffer(i);
    let inv = inverter(buf);
    o = and(i, inv);
  }
  //1
  pub fn tri_state(i, e) -> (o) {
    let (dir, inv) = bin_deco(i, e);
    o = snowflake(dir, inv);
  }
  pub fn d_latch(i, e) -> (q, qn) {
    let (dir, inv) = bin_deco(i, e);
    (q, qn) = sr_latch(dir, inv);
  }
  pub fn jk_latch(j, k, e) -> (q, qn) {
    let s0 = and(qn, e);
    let s = and(s0, j);
    let r0 = and(q, e);
    let r = and(r0, k);
    (q, qn) = sr_latch(s, r);
  }
  pub fn half_adder(i0, i1) -> (s, c) {
    s = xor(i0, i1);
    c = and(i0, i1);
  }
  //2
  pub fn d_flip_flop(i, clk) -> (q, qn) {
    let e = rising_edge(clk);
    (q, qn) = d_latch(i, e);
  }
  pub fn jk_flip_flop(j, k, clk) -> (q, qn) {
    let e = rising_edge(clk);
    (q, qn) = jk_latch(j, k, e);
  }
  pub fn full_adder(i0, i1, c_in) -> (s, c_out) {
    let (sum, c0) = half_adder(i0, i1);
    let c1 = and(sum, c_in);
    s = xor(sum, c_in);
    c_out = and(c0, c1);
  }
  //3
}
