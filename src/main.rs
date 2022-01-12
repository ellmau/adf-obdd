pub mod adf;
pub mod datatypes;
pub mod obdd;
pub mod parser;

use std::{fs::File, path::PathBuf};

use adf::Adf;

use clap::{crate_authors, crate_description, crate_name, crate_version};
use parser::AdfParser;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = crate_name!(), about = crate_description!(), author = crate_authors!(), version = crate_version!())]
struct App {
    /// Input filename
    #[structopt(parse(from_os_str))]
    input: PathBuf,
    /// Sets the verbosity to 'warn', 'info', 'debug' or 'trace' if -v and -q are not use
    #[structopt(long = "rust_log", env)]
    rust_log: Option<String>,
    /// Sets log verbosity (multiple times means more verbose)
    #[structopt(short, parse(from_occurrences), group = "verbosity")]
    verbose: u8,
    /// Sets log verbosity to only errors
    #[structopt(short, group = "verbosity")]
    quiet: bool,
    /// Sorts variables in an lexicographic manner
    #[structopt(long = "lx", group = "sorting")]
    sort_lex: bool,
    /// Sorts variables in an alphanumeric manner
    #[structopt(long = "an", group = "sorting")]
    sort_alphan: bool,
    /// Compute the grounded model
    #[structopt(long = "grd")]
    grounded: bool,
    /// Compute the stable models
    #[structopt(long = "stm")]
    stable: bool,
    /// Compute the complete models
    #[structopt(long = "com")]
    complete: bool,
    /// Import an adf- bdd state instead of an adf
    #[structopt(long)]
    import: bool,
    /// Export the adf-bdd state after parsing and BDD instantiation to the given filename
    #[structopt(long)]
    export: Option<PathBuf>,
}

impl App {
    fn run(&self) {
        let filter_level = match self.verbose {
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            3 => log::LevelFilter::Trace,
            _ => {
                if self.quiet {
                    log::LevelFilter::Error
                } else if let Some(rust_log) = self.rust_log.clone() {
                    match rust_log.as_str() {
                        "error" => log::LevelFilter::Error,
                        "info" => log::LevelFilter::Info,
                        "debug" => log::LevelFilter::Debug,
                        "trace" => log::LevelFilter::Trace,
                        _ => log::LevelFilter::Warn,
                    }
                } else {
                    log::LevelFilter::Warn
                }
            }
        };
        env_logger::builder().filter_level(filter_level).init();
        log::info!("Version: {}", crate_version!());
        let input = std::fs::read_to_string(self.input.clone()).expect("Error Reading File");
        let mut adf = if self.import {
            serde_json::from_str(&input).unwrap()
        } else {
            let parser = AdfParser::default();
            parser.parse()(&input).unwrap();
            log::info!("[Done] parsing");
            if self.sort_lex {
                parser.varsort_lexi();
            }
            if self.sort_alphan {
                parser.varsort_alphanum();
            }
            Adf::from_parser(&parser)
        };
        if let Some(export) = &self.export {
            if export.exists() {
                log::error!(
                    "Cannot write JSON file <{}>, as it already exists",
                    export.to_string_lossy()
                );
            } else {
                let export_file = match File::create(&export) {
                    Err(reason) => {
                        panic!("couldn't create {}: {}", export.to_string_lossy(), reason)
                    }
                    Ok(file) => file,
                };
                serde_json::to_writer(export_file, &adf).unwrap_or_else(|_| {
                    panic!("Writing JSON file {} failed", export.to_string_lossy())
                });
            }
        }
        if self.grounded {
            let grounded = adf.grounded();
            print!("{}", adf.print_interpretation(&grounded));
        }
        if self.complete {
            let complete = adf.complete(0);
            for model in complete {
                print!("{}", adf.print_interpretation(&model));
            }
        }
        if self.stable {
            let stable = adf.stable(0);
            for model in stable {
                print!("{}", adf.print_interpretation(&model));
            }
        }
    }
}

fn main() {
    let app = App::from_args();
    app.run();
}
