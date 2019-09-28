use std::io::{stdin, BufRead};
use std::convert::TryInto;
use circuit_sim::base::WholeNewState;
use circuit_sim::circuit::*;
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
  let mut args = std::env::args().skip(1);
  let path = args.next().unwrap();
  let func_name = args.next().unwrap();
  let (funcs, env) = ast::mir::parse(&std::fs::read_to_string(path).unwrap()).unwrap_or_else(|e|panic!(e));
  //println!("{:#?}", env);
  let mut circuit = funcs[env[&func_name].0].build_circuit(&funcs);
  //println!("{:#?}", circuit);
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
