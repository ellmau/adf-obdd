use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;
use std::usize;
use std::{env, process::exit};

pub mod adf;
pub mod obdd;

use adf::Adf;

fn main() {
    let start_time = Instant::now();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("No Filename given");
        exit(1);
    }
    let mut statements: Vec<String> = Vec::new();
    let mut ac: Vec<(String, String)> = Vec::new();
    let path = Path::new(args[1].as_str());
    if let Ok(lines) = read_lines(path) {
        for line in lines.flatten() {
            //if let Ok(line) = resline {
            //let slice = line.as_str();
            if line.starts_with("s(") {
                // let slice = line.as_str();
                // statements.push(Adf::findterm_str(&slice[2..]).clone());
                statements
                    .push(Adf::findterm_str(line.strip_prefix("s(").unwrap()).replace(" ", ""));
            } else if line.starts_with("ac(") {
                let (s, c) = Adf::findpairs(line.strip_prefix("ac(").unwrap());
                ac.push((s.replace(" ", ""), c.replace(" ", "")));
            }
            //}
        }
    }

    let file_read = start_time.elapsed();
    let start_shortcut = Instant::now();

    println!("parsed {} statements after {}ms", statements.len(), file_read.as_millis());
    if !statements.is_empty() && !ac.is_empty() {
        let mut my_adf = Adf::new();
        my_adf.init_statements(statements.iter().map(AsRef::as_ref).collect());
        for (s, c) in ac.clone() {
            my_adf.add_ac(s.as_str(), c.as_str());
        }

        let result = my_adf.grounded();
        //println!("{:?}",result);
        // for (p, s) in statements.iter().enumerate() {
        //     match result[p] {
        //         0 => print!("f("),
        //         1 => print!("t("),
        //         _ => print!("u("),
        //     }
        //     println!("{}) ", *s);
        // }
        print_interpretation(result);
        println!("finished after {}ms", start_shortcut.elapsed().as_millis());
        let start_fp = Instant::now();

        let mut my_adf2 = Adf::default();
        my_adf2.init_statements(statements.iter().map(AsRef::as_ref).collect());
        for (s, c) in ac {
            my_adf2.add_ac(s.as_str(), c.as_str());
        }
        let empty_int = my_adf2.cur_interpretation().to_owned();
        let result2 = my_adf2.to_fixpoint(empty_int).unwrap();
        //   for (p, s) in statements.iter().enumerate() {
        //     match result2[p] {
        //         0 => print!("f("),
        //         1 => print!("t("),
        //         _ => print!("u("),
        //     }
        //     println!("{}) ", *s);
        // }
        print_interpretation(result2);
        println!("finished after {}ms", start_fp.elapsed().as_millis());
    }
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn print_interpretation(interpretation: Vec<usize>) {
    for it in interpretation.iter() {
        match *it {
            0 => print!("f"),
            1 => print!("t"),
            _ => print!("u"),
        }
    }
    println!("");
}
