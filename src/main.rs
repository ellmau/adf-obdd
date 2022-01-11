pub mod adf;
pub mod datatypes;
pub mod obdd;
pub mod parser;

use std::str::FromStr;

use adf::Adf;

use clap::{clap_app, crate_authors, crate_description, crate_name, crate_version};
use parser::AdfParser;

fn main() {
    let matches = clap_app!(myapp =>
                (version: crate_version!())
                (author: crate_authors!())
                (name: crate_name!())
                (about: crate_description!())
                //(@arg fast: -f --fast "fast algorithm instead of the direct fixpoint-computation")
                (@arg verbose: -v +multiple "Sets log verbosity")
                (@arg INPUT: +required "Input file")
                (@group sorting =>
                 (@arg sort_lex: --lx "Sorts variables in a lexicographic manner")
                 (@arg sort_alphan: --an "Sorts variables in an alphanumeric manner")
                )
                (@arg grounded: --grd "Compute the grounded model")
                (@arg stable: --stm "Compute the stable models")
    )
    .get_matches_safe()
    .unwrap_or_else(|e| match e.kind {
        clap::ErrorKind::HelpDisplayed => {
            e.exit();
        }
        clap::ErrorKind::VersionDisplayed => {
            e.exit();
        }
        _ => {
            eprintln!("{} Version {{{}}}", crate_name!(), crate_version!());
            e.exit();
        }
    });
    let filter_level = match matches.occurrences_of("verbose") {
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        3 => log::LevelFilter::Trace,
        _ => {
            match std::env::vars().find_map(|(var, val)| {
                if var.eq("RUST_LOG") {
                    Some(log::LevelFilter::from_str(val.as_str()))
                } else {
                    None
                }
            }) {
                Some(v) => v.unwrap_or(log::LevelFilter::Error),
                None => log::LevelFilter::Error,
            }
        }
    };
    env_logger::builder().filter_level(filter_level).init();
    log::info!("Version: {}", crate_version!());

    let input = std::fs::read_to_string(
        matches
            .value_of("INPUT")
            .expect("Input Filename should be given"),
    )
    .expect("Error Reading File");
    let parser = AdfParser::default();
    parser.parse()(&input).unwrap();
    log::info!("[Done] parsing");

    if matches.is_present("sort_lex") {
        parser.varsort_lexi();
    }
    if matches.is_present("sort_alphan") {
        parser.varsort_alphanum();
    }

    let mut adf = Adf::from_parser(&parser);
    if matches.is_present("grounded") {
        let grounded = adf.grounded();
        println!("{}", adf.print_interpretation(&grounded));
    }
    if matches.is_present("stable") {
        let stable = adf.stable(1);
        for model in stable {
            println!("{}", adf.print_interpretation(&model));
        }
    }
}
