use std::{env, process::exit};
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

pub mod obdd;
pub mod adf;

use adf::Adf;

fn main() {
  let args:Vec<String> = env::args().collect();
  if args.len() != 2 {
    eprintln!("No Filename given");
    exit(1);
  }
  let mut statements: Vec<String> = Vec::new();
  let mut ac: Vec<(String,String)> = Vec::new();
  let path = Path::new(args[1].as_str());
  if let Ok(lines) = read_lines(path){
    for line in lines.flatten() {
      //if let Ok(line) = resline {
        //let slice = line.as_str();
        if line.starts_with("s("){
         // let slice = line.as_str();
         // statements.push(Adf::findterm_str(&slice[2..]).clone());
          statements.push(Adf::findterm_str(line.strip_prefix("s(").unwrap()).replace(" ", ""));
        }
        else if line.starts_with("ac("){      
          let (s,c) = Adf::findpairs(line.strip_prefix("ac(").unwrap());
          ac.push((s.replace(" ",""),c.replace(" ", "")));
        }
      //}
    }
  }

  println!("parsed {} statements", statements.len());
  if !statements.is_empty() && !ac.is_empty() {
    let mut my_adf = Adf::new();
    my_adf.init_statements(statements.iter().map(AsRef::as_ref).collect());
    for (s,c) in ac {
      my_adf.add_ac(s.as_str(), c.as_str());
    }

    let result = my_adf.grounded();
    println!("{:?}",result);
    for (p,s) in statements.iter().enumerate(){
      match result[p] {
        0 => print!("f("),
        1 => print!("t("),
        _ => print!("u("),
      }
      print!("{}) ",*s);
    }

  }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}