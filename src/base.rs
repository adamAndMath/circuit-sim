use rand::Rng;
use std::path::Path;
use std::fs::{ read_to_string, write };

pub type Data = bool;
type ID = usize;
#[derive(Debug)]
pub enum Component {
  Source(Data),
  Buffer(usize),
  Inverter(usize),
  Or(usize, usize),
  And(usize, usize),
  Nor(usize, usize),
  Nand(usize, usize),
  Bus(Vec<(usize, usize)>),
}
impl Component {
  fn update(&self, wires: &[Data]) -> Data {
    match *self {
      Component::Source(out) => out,
      Component::Buffer(in0) => wires[in0],
      Component::Inverter(in0) => !wires[in0],
      Component::Or(in0, in1) => wires[in0] || wires[in1],
      Component::And(in0, in1) => wires[in0] && wires[in1],
      Component::Nor(in0, in1) => !(wires[in0] || wires[in1]),
      Component::Nand(in0, in1) => !(wires[in0] && wires[in1]),
      Component::Bus(ref inputs) => {
        let mut up: bool = false;
        let mut down: bool = false;
        for (s_up, s_down) in inputs.iter() {
          up |= wires[*s_up];
          down |= wires[*s_down];
        }
        match (up, down) {
          (false, true) => false,
          (true, false) => true,
          _ => rand::thread_rng().gen(),
        }
      },
    }
  }
}
#[derive(Default, Debug)]
pub struct WholeNew {
  pub components: Box<[(Component, Data)]>,
}
#[derive(Debug)]
pub struct WholeNewState {
  pub components: Box<[Data]>,
  old_components: Box<[Data]>,
}
impl WholeNew {
  pub fn new_state(&self) -> WholeNewState {
    let components = self.components.iter().map(|(_,p)|p).cloned().collect::<Vec<_>>().into_boxed_slice();
    WholeNewState { old_components: components.clone(), components }
  }
  pub fn update(&self, state: &mut WholeNewState) {
    std::mem::swap(&mut state.components, &mut state.old_components);
    for (comp, out) in self.components.iter().map(|(comp,_)|comp).zip(state.components.iter_mut()) {
      *out = comp.update(&state.old_components);
    }
  }
}
impl WholeNewState {
  pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
    write(path, self.components.iter().map(|b|if *b { b'1' } else { b'0' }).collect::<Vec<_>>())
  }
  pub fn load<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
    let file = read_to_string(path)?;
    if file.len() != self.components.len() {
      panic!();
    }
    self.components.iter_mut().zip(file.chars().map(|c|match c { '1' => true, '0' => false, _ => panic!() })).for_each(|(wire, val)|*wire = val);
    self.old_components.copy_from_slice(&self.components);
    Ok(())
  }
}
