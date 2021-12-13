use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::time::Instant;
use std::usize;

pub mod adf;
pub mod datatypes;
pub mod obdd;
pub mod parser;

use adf::Adf;

#[macro_use]
extern crate clap;

fn main() {
    let matches = clap_app!(myapp =>
    (version: crate_version!())
    (author: crate_authors!())
    (name: crate_name!())
    (about: crate_description!())
    (@arg fast: -f --fast "fast algorithm instead of the direct fixpoint-computation")
    (@arg verbose: -v +multiple "Sets verbosity")
    (@arg INPUT: +required "Input file")
    )
    .get_matches();
    let verbose = matches.occurrences_of("verbose") > 0;
    let start_time = Instant::now();
    //let args: Vec<String> = env::args().collect();
    //if args.len() != 2 {
    //    eprintln!("No Filename given");
    //    exit(1);
    //}
    let mut statements: Vec<String> = Vec::new();
    let mut ac: Vec<(String, String)> = Vec::new();
    let path = Path::new(matches.value_of("INPUT").unwrap());
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

    if verbose {
        println!(
            "parsed {} statements after {}ms",
            statements.len(),
            file_read.as_millis()
        );
    }

    if !statements.is_empty() && !ac.is_empty() {
        if matches.is_present("fast") {
            let mut my_adf = Adf::new();
            my_adf.init_statements(statements.iter().map(AsRef::as_ref).collect());
            for (s, c) in ac.clone() {
                my_adf.add_ac(s.as_str(), c.as_str());
            }

            let result = my_adf.grounded();
            print_interpretation(result);
            if verbose {
                println!("finished after {}ms", start_shortcut.elapsed().as_millis());
            }
        } else {
            let start_fp = Instant::now();

            let mut my_adf = Adf::default();
            my_adf.init_statements(statements.iter().map(AsRef::as_ref).collect());
            for (s, c) in ac.clone() {
                my_adf.add_ac(s.as_str(), c.as_str());
            }
            let empty_int = my_adf.cur_interpretation();
            let result = my_adf.compute_fixpoint(empty_int.as_ref()).unwrap();

            print_interpretation(result);
            if verbose {
                println!("finished after {}ms", start_fp.elapsed().as_millis());
            }
        }
        // optional test of complete extensions
        // let mut my_adf3 = Adf::default();
        // my_adf3.init_statements(statements.iter().map(AsRef::as_ref).collect());
        // for (s, c) in ac.clone() {
        //     my_adf3.add_ac(s.as_str(), c.as_str());
        // }

        // let result3 = my_adf3.complete();
        // for it in result3.iter() {
        //     print_interpretation(it.to_vec());
        // }
        // println!("{}",result3.len());
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
    let mut stable = true;
    for it in interpretation.iter() {
        match *it {
            0 => print!("f"),
            1 => print!("t"),
            _ => {
                print!("u");
                stable = false
            }
        }
    }
    if stable {
        println!(" stm");
    } else {
        println!();
    }
}
