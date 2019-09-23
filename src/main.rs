use std::io::{stdin, BufRead};
use std::convert::TryInto;
use circuit_sim::base::WholeNewState;
use circuit_sim::circuit::*;
use circuit_sim::circuit;

fn parse_bool(s: char) -> Result<bool, String> {
  match s {
    '0' => Ok(false),
    '1' => Ok(true),
    s => Err(format!("Undefined value: {}", s))
  }
}
fn run_command(circuit: &mut Circuit, state: &mut WholeNewState, cmd: &str, args: &[&str]) -> Result<bool, String> {
  match cmd {
    "exit" => Ok(true),
    "set" => {
      let [arg]: [&str; 1] = args.try_into().map_err(|_|format!("Expected 1 argument, recieved {}", args.len()))?;
      let input = arg.chars().map(parse_bool).collect::<Result<_, String>>()?;
      circuit.set_input(input)?;
      Ok(false)
    },
    "run" => {
      let [arg]: [&str; 1] = args.try_into().map_err(|_|format!("Expected 1 argument, recieved {}", args.len()))?;
      let steps = arg.parse().map_err(|_|format!("Not a number: {}", arg))?;
      for _ in 0..steps {
        circuit.update(state);
        circuit.print_output(state);
      }
      Ok(false)
    },
    cmd => Err(format!("Unknown command: {}", cmd)),
  }
}
fn main() {
  let mut circuit = Circuit::default();
  circuit! { &mut circuit;
    let reset = input(false);
    let set = input(true);
    let (q, qn) = flip_flop(reset, set);
    output(q);
    output(qn);
  }
  let mut state = circuit.new_state();
  for line in stdin().lock().lines().map(|l|l.unwrap()) {
    let args: Vec<&str> = line.split_whitespace().collect();
    let (cmd, args) = match args.split_first() {
      Some(cmd) => cmd,
      None => continue,
    };
    match run_command(&mut circuit, &mut state, cmd, &args) {
      Ok(b) => if b { return },
      Err(err) => println!("{}", err),
    }
  }
}
