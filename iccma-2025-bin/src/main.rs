use adf_bdd::adf::Adf;
use adf_bdd::parser::{AdfParser, Formula};
use clap::{builder, Parser, ValueEnum};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::BufRead;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(ValueEnum, Clone)]
enum Task {
    #[value(name = "DC-CO")]
    DcCo,
    #[value(name = "DC-ST")]
    DcSt,
    #[value(name = "DC-SST")]
    DcSst,
    #[value(name = "DS-PR")]
    DsPr,
    #[value(name = "DS-ST")]
    DsSt,
    #[value(name = "DS-SST")]
    DsSst,
    #[value(name = "SE-PR")]
    SePr,
    #[value(name = "SE-ST")]
    SeSt,
    #[value(name = "SE-SST")]
    SeSst,
    #[value(name = "SE-ID")]
    SeId,
}

#[derive(Parser)]
#[command(
    author,
    version,
    arg_required_else_help(true),
    help_template("{name} {version}\n{author-with-newline}")
)]
struct App {
    #[arg(long, exclusive(true))]
    problems: bool,
    #[arg(short = 'p', value_enum, required_unless_present("problems"))]
    task: Option<Task>,
    #[arg(short = 'f', value_parser, required_unless_present("problems"))]
    input_file: Option<PathBuf>,
    #[arg(short = 'a', required_if_eq_any([("task", "DC-CO"), ("task", "DC-ST"), ("task", "DC-SST"), ("task", "DS-PR"), ("task", "DS-ST"), ("task", "DS-SST")]))]
    query: Option<usize>,
}

struct AF(Vec<Vec<usize>>);

impl From<AF> for AdfParser {
    fn from(source: AF) -> Self {
        let names: Vec<String> = (0..source.0.len())
            .map(|val| (val + 1).to_string())
            .collect();
        let dict: HashMap<String, usize> = names
            .iter()
            .enumerate()
            .map(|(i, val)| (val.clone(), i))
            .collect();
        let formulae: Vec<Formula> = source
            .0
            .into_iter()
            .map(|attackers| {
                attackers.into_iter().fold(
                    // TODO: is it correct to use Top here? what if something is not attacked at all?
                    Formula::Top,
                    |acc, attacker| {
                        Formula::And(
                            Box::new(acc),
                            Box::new(Formula::Not(Box::new(Formula::Atom(
                                (attacker + 1).to_string(),
                            )))),
                        )
                    },
                )
            })
            .collect();
        let formulanames = names.clone();

        Self {
            namelist: Arc::new(RwLock::new(names)),
            dict: Arc::new(RwLock::new(dict)),
            formulae: RefCell::new(formulae),
            formulaname: RefCell::new(formulanames),
        }
    }
}

fn main() {
    let app = App::parse();

    if app.problems {
        let possible_values: Vec<String> = Task::value_variants()
            .into_iter()
            .filter_map(Task::to_possible_value)
            .map(|pv| builder::PossibleValue::get_name(&pv).to_string())
            .collect();
        print!("[");
        print!("{}", possible_values.join(","));
        println!("]")
    } else {
        let task = app
            .task
            .expect("Task is required when \"problems\" flag is false.");
        let file = app
            .input_file
            .expect("File is required when \"problems\" flag is false.");

        let file = std::fs::File::open(file).expect("Error Reading File");
        let mut lines = std::io::BufReader::new(file).lines();

        let first_line = lines
            .next()
            .expect("There must be at least one line in the file")
            .expect("Error Reading Line");
        let first_line: Vec<_> = first_line.split(" ").collect();
        if first_line[0] != "p" || first_line[1] != "af" {
            panic!("Expected first line to be of the form: p af <n>");
        }

        let num_arguments: usize = first_line[2].parse().expect("Could not convert number of arguments to u32; expected first line to be of the form: p af <n>");

        let attacks: Vec<(usize, usize)> = lines
            .map(|line| line.expect("Error Reading Line"))
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(|line| {
                let mut line = line.split(" ");
                let a = line.next()?;
                let b = line.next()?;
                if line.next().is_some() {
                    None
                } else {
                    Some((a.parse().ok()?, b.parse().ok()?))
                }
            })
            .map(|res_option| res_option.expect("Line must be of the form: n m"))
            .collect();

        // index in outer vector represents attacked element
        let mut is_attacked_by: Vec<Vec<usize>> = vec![vec![]; num_arguments.try_into().unwrap()];
        for (a, b) in attacks {
            is_attacked_by[b - 1].push(a - 1); // we normalize names to be zero-indexed
        }

        let hacked_adf_parser = AdfParser::from(AF(is_attacked_by));

        let mut adf = Adf::from_parser(&hacked_adf_parser);

        match task {
            Task::DcCo => {
                unimplemented!()
                //let query: usize = app.query.expect("Query is required for current task.");
                //let printer = adf.print_dictionary();
                //let mut models = adf.complete();
                //if let Some(fst) = models.filter(|m| {
                //    // TODO: filter for query here; use dict somehow
                //}).next() {
                //    println!("YES");
                //    println!("w {}", printer.print_truthy_statements(&fst).join(" "));
                //}
                //else {
                //    println!("NO");
                //}
            }
            Task::DcSt => {
                unimplemented!()
            }
            Task::DcSst => {
                unimplemented!()
            }
            Task::DsPr => {
                unimplemented!()
            }
            Task::DsSt => {
                unimplemented!()
            }
            Task::DsSst => {
                unimplemented!()
            }
            Task::SePr => {
                let printer = adf.print_dictionary();
                let models = adf.preferred();
                if let Some(fst) = models.first() {
                    println!("w {}", printer.print_truthy_statements(&fst).join(" "));
                } else {
                    println!("NO");
                }
            }
            Task::SeSt => {
                let printer = adf.print_dictionary();
                let mut models = adf.stable();
                if let Some(fst) = models.next() {
                    println!("w {}", printer.print_truthy_statements(&fst).join(" "));
                } else {
                    println!("NO");
                }
            }
            Task::SeSst => {
                // from my understanding, semi-stable is the same as grounded
                let printer = adf.print_dictionary();
                let model = adf.grounded();
                println!("w {}", printer.print_truthy_statements(&model).join(" "));
            }
            Task::SeId => {
                unimplemented!()
            }
        }
    }
}
