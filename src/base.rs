use rand::Rng;

type Source = bool;
type Point = Option<bool>;
#[derive(Default)]
pub struct Wire {
  pub inputs: Vec<usize>,
}
impl Wire {
  fn update(&self, components: &[Point]) -> Source {
    let mut up: bool = false;
    let mut down: bool = false;
    for point in self.inputs.iter() {
      if let Some(signal) = components[*point] {
        if signal {
          up = true;
        } else {
          down = true;
        }
      }
    }
    // for signal in self.inputs.iter().filter_map(|x|components[*x]) {
    //       if signal {
    //         up = true;
    //       } else {
    //         down = true;
    //       }
    // }
    match (up, down) {
      (false, true) => false,
      (true, false) => true,
      _ => rand::thread_rng().gen(),
    }
  }
}
pub enum Component {
  Source(Source),
  Buffer(usize),
  Inverter(usize),
  Or(usize, usize),
  And(usize, usize),
  Nor(usize, usize),
  Nand(usize, usize),
  Snowflake(usize, usize),
}
impl Component {
  fn update(&self, wires: &[Source]) -> Point {
    match *self {
      Component::Source(out) => Some(out),
      Component::Buffer(in0) => Some(wires[in0]),
      Component::Inverter(in0) => Some(!wires[in0]),
      Component::Or(in0, in1) => Some(wires[in0] || wires[in1]),
      Component::And(in0, in1) => Some(wires[in0] && wires[in1]),
      Component::Nor(in0, in1) => Some(!(wires[in0] || wires[in1])),
      Component::Nand(in0, in1) => Some(!(wires[in0] && wires[in1])),
      Component::Snowflake(in0, in1) => match (wires[in0], wires[in1]) {
        (false, true) => Some(false),
        (true, false) => Some(true),
        _ => None,
      },
    }
  }
}
#[derive(Default)]
pub struct WholeNew {
  pub components: Vec<Component>,
  pub wires: Vec<Wire>,
}
#[derive(Debug)]
pub struct WholeNewState {
  pub components: Vec<Point>,
  pub wires: Vec<Source>,
}
impl WholeNew {
  pub fn new_state(&self) -> WholeNewState {
    WholeNewState {
      wires: vec![false; self.wires.len()],
      components: vec![None; self.components.len()],
    }
  }
  pub fn update(&self, state: &mut WholeNewState) {
    for (comp, out) in self.components.iter().zip(state.components.iter_mut()) {
      *out = comp.update(&state.wires);
    }
    for (wire, out) in self.wires.iter().zip(state.wires.iter_mut()) {
      *out = wire.update(&state.components);
    }
  }
}
