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
