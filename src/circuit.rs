use crate::base::{Data, Component, WholeNew, WholeNewState};
use crate::slot_vec::SlotVec;

pub struct Builder {
  components: SlotVec<(Component, Data)>,
  inputs: Vec<usize>,
  outputs: Vec<usize>,
}

impl Builder {
  pub fn new_slot(&mut self) -> usize {
    self.components.new_slot()
  }
  pub fn place_component(&mut self, slot: usize, component: Component, default: bool) {
    self.components.fill_slot(slot, (component, default));
  }
  pub fn add_input(&mut self, slot: usize, default: bool) {
    self.place_component(slot, Component::Source(default), default);
    self.inputs.push(slot);
  }
  pub fn add_output(&mut self, index: usize) {
    self.outputs.push(index)
  }
  pub fn add_bus_input(&mut self, bus: usize, high: usize, low: usize) {
    match &mut self.components[bus] {
      (Component::Bus(inputs), _) => inputs.push((high, low)),
      _ => panic!("Not a bus"),
    }
  }
  pub fn build(self) -> Circuit {
    Circuit {
      whole_new: WholeNew { components: self.components.build() },
      inputs: self.inputs.into_boxed_slice(),
      outputs: self.outputs.into_boxed_slice(),
    }
  }
}

#[derive(Debug)]
pub struct Circuit {
  whole_new: WholeNew,
  inputs: Box<[usize]>,
  outputs: Box<[usize]>,
}

impl Circuit {
  pub fn builder() -> Builder {
    Builder {
      components: SlotVec::new(),
      inputs: Vec::new(),
      outputs: Vec::new(),
    }
  }
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
    self.outputs.iter().copied().map(|i|state.components[i]).for_each(|b|if b { print!("1") } else { print!("0") });
    println!();
  }
}
#[must_use]
pub fn component(component: Component, default: bool) -> impl FnOnce(&mut Builder, usize) {
  move |circuit, o| circuit.place_component(o, component, default)
}
#[must_use]
pub fn input(default: bool) -> impl FnOnce(&mut Builder, usize) {
  move |circuit, o| circuit.add_input(o, default)
}
#[must_use]
pub fn output(i: usize) -> impl FnOnce(&mut Builder) {
  move |circuit| circuit.add_output(i)
}
#[must_use]
pub fn bus_input(bus: usize, high: usize, low: usize) -> impl FnOnce(&mut Builder) {
  move |circuit| circuit.add_bus_input(bus, high, low)
}
define! {
  pub fn buffer(i) -> (o) {
    o = component(Component::Buffer(i), false);
  }
  pub fn inverter(i) -> (o) {
    o = component(Component::Inverter(i), true);
  }
  pub fn or(i0, i1) -> (o) {
    o = component(Component::Or(i0, i1), false);
  }
  pub fn and(i0, i1) -> (o) {
    o = component(Component::And(i0, i1), false);
  }
  pub fn nor(i0, i1) -> (o) {
    o = component(Component::Nor(i0, i1), true);
  }
  pub fn nand(i0, i1) -> (o) {
    o = component(Component::Nand(i0, i1), true);
  }
  fn bus() -> (o) {
    o = component(Component::Bus(vec![]), false);
  }
}

pub fn clock(n: usize) -> impl FnOnce(&mut Builder, usize) {
  move |circuit, o| {
    let mut wire = o;
    for _ in 1..n {
      let f = buffer(wire);
      wire = circuit.new_slot();
      f(circuit, wire);
    }
    inverter(wire)(circuit, o);
  }
}

define! {
  //1
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
  pub fn select(i0, i1, s) -> (o) {
    let sn = inverter(s);
    let o0 = and(s, i0);
    let o1 = and(sn, i1);
    o = or(o0, o1);
  }
  pub fn bin_deco(i, e) -> (dir, inv) {
    let not = inverter(i);
    dir = and(e, i);
    inv = and(e, not);
  }
  pub fn sr_latch(s, r) -> (q, qn) {
    q = component(Component::Nor(r, qn), false);
    qn = nor(s, q);
  }
  pub fn rising_edge(i) -> (o) {
    let buf = buffer(i);
    let inv = inverter(buf);
    o = and(i, inv);
  }
  //2
  pub fn tri_state(i, e, bus) -> () {
    let (dir, inv) = bin_deco(i, e);
    bus_input(bus, dir, inv);
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
  pub fn inverter8(i0, i1, i2, i3, i4, i5, i6, i7, inv) -> (o0, o1, o2, o3, o4, o5, o6, o7) {
    o0 = xor(i0, inv);
    o1 = xor(i1, inv);
    o2 = xor(i2, inv);
    o3 = xor(i3, inv);
    o4 = xor(i4, inv);
    o5 = xor(i5, inv);
    o6 = xor(i6, inv);
    o7 = xor(i7, inv);
  }
  //3
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
    c_out = or(c0, c1);
  }
  pub fn register8(i0, i1, i2, i3, i4, i5, i6, i7, load, clk) -> (o0, o1, o2, o3, o4, o5, o6, o7) {
    let e = and(load, clk);
    let (n0, n1, n2, n3, n4, n5, n6, n7);
    (o0, n0) = d_latch(i0, e);
    (o1, n1) = d_latch(i1, e);
    (o2, n2) = d_latch(i2, e);
    (o3, n3) = d_latch(i3, e);
    (o4, n4) = d_latch(i4, e);
    (o5, n5) = d_latch(i5, e);
    (o6, n6) = d_latch(i6, e);
    (o7, n7) = d_latch(i7, e);
  }
  pub fn tri_state8(i0, i1, i2, i3, i4, i5, i6, i7, e, bus0, bus1, bus2, bus3, bus4, bus5, bus6, bus7) -> () {
    tri_state(i0, e, bus0);
    tri_state(i1, e, bus1);
    tri_state(i2, e, bus2);
    tri_state(i3, e, bus3);
    tri_state(i4, e, bus4);
    tri_state(i5, e, bus5);
    tri_state(i6, e, bus6);
    tri_state(i7, e, bus7);
  }
  //4
  pub fn full_adder8(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7, c_in) -> (o0, o1, o2, o3, o4, o5, o6, o7, c_out) {
    let (c0, c1, c2, c3, c4, c5, c6);
    (o0, c0) = full_adder(a0, b0, c_in);
    (o1, c1) = full_adder(a1, b1, c0);
    (o2, c2) = full_adder(a2, b2, c1);
    (o3, c3) = full_adder(a3, b3, c2);
    (o4, c4) = full_adder(a4, b4, c3);
    (o5, c5) = full_adder(a5, b5, c4);
    (o6, c6) = full_adder(a6, b6, c5);
    (o7, c_out) = full_adder(a7, b7, c6);
  }
  //5
  pub fn ALU(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7, sign, e, bus0, bus1, bus2, bus3, bus4, bus5, bus6, bus7) -> () {
    let (b0, b1, b2, b3, b4, b5, b6, b7) = inverter8(b0, b1, b2, b3, b4, b5, b6, b7, sign);
    let (c0, c1, c2, c3, c4, c5, c6, c7, c_out) = full_adder8(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7, sign);
    tri_state8(c0, c1, c2, c3, c4, c5, c6, c7, e, bus0, bus1, bus2, bus3, bus4, bus5, bus6, bus7);
  }
}
