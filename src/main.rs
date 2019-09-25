use std::io::{stdin, BufRead};
use std::convert::TryInto;
use circuit_sim::base::WholeNewState;
use circuit_sim::circuit::*;
use circuit_sim::circuit;
mod env;
mod ast;

fn parse_bool(s: char) -> Result<bool, String> {
  match s {
    '0' => Ok(false),
    '1' => Ok(true),
    s => Err(format!("Undefined value: {}", s))
  }
}
fn run_command(circuit: &mut Circuit, state: &mut WholeNewState, cmd: &str, args: &[&str]) -> Result<bool, String> {
  match cmd {
    "set" => {
      let [arg]: [&str; 1] = args.try_into().map_err(|_|format!("Expected 1 argument, recieved {}", args.len()))?;
      let input = arg.chars().map(parse_bool).collect::<Result<_, String>>()?;
      circuit.set_input(input)?;
    },
    "run" => {
      let [arg]: [&str; 1] = args.try_into().map_err(|_|format!("Expected 1 argument, recieved {}", args.len()))?;
      let steps = arg.parse().map_err(|_|format!("Not a number: {}", arg))?;
      for _ in 0..steps {
        circuit.update(state);
        circuit.print_output(state);
      }
    },
    "save" => {
      let [path]: [&str; 1] = args.try_into().map_err(|_|format!("Expected 1 argument, recieved {}", args.len()))?;
      state.save(path).map_err(|e|format!("{}", e))?;
    },
    "load" => {
      let [path]: [&str; 1] = args.try_into().map_err(|_|format!("Expected 1 argument, recieved {}", args.len()))?;
      state.load(path).map_err(|e|format!("{}", e))?;
    },
    "exit" => return Ok(true),
    cmd => return Err(format!("Unknown command: {}", cmd)),
  }
  Ok(false)
}
fn main() {
  let mut circuit = Circuit::default();
  circuit! { &mut circuit;
    let clk = clock(5);
    let a7 = input(false);
    let a6 = input(false);
    let a5 = input(false);
    let a4 = input(false);
    let a3 = input(false);
    let a2 = input(false);
    let a1 = input(false);
    let a0 = input(false);
    let b7 = input(false);
    let b6 = input(false);
    let b5 = input(false);
    let b4 = input(false);
    let b3 = input(false);
    let b2 = input(false);
    let b1 = input(false);
    let b0 = input(false);
    let sign = input(false);
    let e = input(false);
    let (o0, o1, o2, o3, o4, o5, o6, o7) = ALU(a0, a1, a2, a3, a4, a5, a6, a7, b0, b1, b2, b3, b4, b5, b6, b7, sign, e);
    output(o7);
    output(o6);
    output(o5);
    output(o4);
    output(o3);
    output(o2);
    output(o1);
    output(o0);
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
