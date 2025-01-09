use std::path::PathBuf;
use std::io::BufRead;
use clap::{builder, Parser, ValueEnum};

#[derive(ValueEnum, Clone)]
enum Task {
    #[value(name="DC-CO")]
    DcCo,
    #[value(name="DC-ST")]
    DcSt,
    #[value(name="DC-SST")]
    DcSst,
    #[value(name="DS-PR")]
    DsPr,
    #[value(name="DS-ST")]
    DsSt,
    #[value(name="DS-SST")]
    DsSst,
    #[value(name="SE-PR")]
    SePr,
    #[value(name="SE-ST")]
    SeSt,
    #[value(name="SE-SST")]
    SeSst,
    #[value(name="SE-ID")]
    SeId,
}

#[derive(Parser)]
#[command(author, version, arg_required_else_help(true), help_template("{name} {version}\n{author-with-newline}"))]
struct App {
    #[arg(long,exclusive(true))]
    problems: bool,
    #[arg(short = 'p', value_enum, required_unless_present("problems"))]
    task: Option<Task>,
    #[arg(short = 'f', value_parser, required_unless_present("problems"))]
    input_file: Option<PathBuf>,
    #[arg(short = 'a', required_if_eq_any([("task", "DC-CO"), ("task", "DC-ST"), ("task", "DC-SST"), ("task", "DS-PR"), ("task", "DS-ST"), ("task", "DS-SST")]))]
    query: Option<usize>,
}

fn main() {
    let app = App::parse();

    if app.problems {
        let possible_values: Vec<String> = Task::value_variants().into_iter().filter_map(Task::to_possible_value).map(|pv| builder::PossibleValue::get_name(&pv).to_string()).collect();
        print!("[");
        print!("{}", possible_values.join(","));
        println!("]")
    } else {
        let task = app.task.expect("Task is required when \"problems\" flag is false.");
        let file = app.input_file.expect("File is required when \"problems\" flag is false.");
        let query = app.query;

        let file = std::fs::File::open(file).expect("Error Reading File");
        let mut lines = std::io::BufReader::new(file).lines();

        let first_line = lines.next().expect("There must be at least one line in the file").expect("Error Reading Line");
        let first_line: Vec<_> = first_line.split(" ").collect();
        if first_line[0] != "p" || first_line[1] != "af" {
            panic!("Expected first line to be of the form: p af <n>");
        }

        let num_arguments: usize = first_line[2].parse().expect("Could not convert number of arguments to u32; expected first line to be of the form: p af <n>");

        let attacks: Vec<(usize, usize)> = lines.map(|line| line.expect("Error Reading Line")).filter(|line| !line.starts_with('#') || line.is_empty()).map(|line| {
            let mut line = line.split(" ");
            let a = line.next()?;
            let b = line.next()?;
            if line.next().is_some() {
                None
            } else {
                Some((a.parse().ok()?, b.parse().ok()?))
            }
        }).map(|res_option| res_option.expect("Line must be of the form: n m")).collect();

        // index in outer vector represents attacked element
        let mut is_attacked_by: Vec<Vec<usize>> = vec![vec![]; num_arguments.try_into().unwrap()];
        for (a, b) in attacks {
            is_attacked_by[b-1].push(a-1); // we normalize names to be zero-indexed
        }
    }

    //app.run();
}
