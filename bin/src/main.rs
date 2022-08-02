/*!
This binary uses an efficient representation of `Abstract Dialectical Frameworks (ADf)` by utilising an implementation of `Ordered Binary Decision Diagrams (OBDD)`

# Abstract Dialectical Frameworks
An `abstract dialectical framework` consists of abstract statements. Each statement has an unique label and might be related to other statements (s) in the ADF. This relation is defined by a so-called acceptance condition (ac), which intuitively is a propositional formula, where the variable symbols are the labels of the statements. An interpretation is a three valued function which maps to each statement a truth value (true, false, undecided). We call such an interpretation a model, if each acceptance condition agrees to the interpration.
# Ordered Binary Decision Diagram
An `ordered binary decision diagram` is a normalised representation of binary functions, where satisfiability- and validity checks can be done relatively cheap.

Note that one advantage of this implementation is that only one oBDD is used for all acceptance conditions. This can be done because all of them have the identical signature (i.e. the set of all statements + top and bottom concepts).
Due to this uniform representation reductions on subformulae which are shared by two or more statements only need to be computed once and is already cached in the data structure for further applications.

The used algorithm to create a BDD, based on a given formula does not perform well on bigger formulae, therefore it is possible to use a state-of-the art library to instantiate the BDD (<https://github.com/sybila/biodivine-lib-bdd>).
It is possible to either stay with the biodivine library or switch back to the variant implemented by adf-bdd.
The variant implemented in this library offers reuse of already done reductions and memoisation techniques, which are not offered by biodivine.
In addition some further features, like counter-model counting is not supported by biodivine.

# Usage
```plain
USAGE:
    adf-bdd [OPTIONS] <INPUT>

ARGS:
    <INPUT>    Input filename

OPTIONS:
        --an                      Sorts variables in an alphanumeric manner
        --com                     Compute the complete models
        --counter <COUNTER>       Set if the (counter-)models shall be computed and printed,
                                  possible values are 'nai' and 'mem' for naive and memoization
                                  repectively (only works in hybrid and naive mode)
        --export <EXPORT>         Export the adf-bdd state after parsing and BDD instantiation to
                                  the given filename
        --grd                     Compute the grounded model
    -h, --help                    Print help information
        --heu <HEU>               Choose which heuristics shall be used by the nogood-learning
                                  approach [possible values: Simple, MinModMinPathsMaxVarImp,
                                  MinModMaxVarImpMinPaths]
        --import                  Import an adf- bdd state instead of an adf
        --lib <IMPLEMENTATION>    Choose the bdd implementation of either 'biodivine', 'naive', or
                                  hybrid [default: hybrid]
        --lx                      Sorts variables in an lexicographic manner
    -q                            Sets log verbosity to only errors
        --rust_log <RUST_LOG>     Sets the verbosity to 'warn', 'info', 'debug' or 'trace' if -v and
                                  -q are not use [env: RUST_LOG=debug]
        --stm                     Compute the stable models
        --stmca                   Compute the stable models with the help of modelcounting using
                                  heuristics a
        --stmcb                   Compute the stable models with the help of modelcounting using
                                  heuristics b
        --stmng                   Compute the stable models with the nogood-learning based approach
        --stmpre                  Compute the stable models with a pre-filter (only hybrid lib-mode)
        --stmrew                  Compute the stable models with a single-formula rewriting (only
                                  hybrid lib-mode)
        --stmrew2                 Compute the stable models with a single-formula rewriting on
                                  internal representation(only hybrid lib-mode)
        --twoval                  Compute the two valued models with the nogood-learning based
                                  approach
    -v                            Sets log verbosity (multiple times means more verbose)
    -V, --version                 Print version information
```
 */

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![warn(
    missing_docs,
    unused_import_braces,
    unused_qualifications,
    unused_extern_crates,
    variant_size_differences
)]

use std::{fs::File, path::PathBuf};

use adf_bdd::adf::Adf;
use adf_bdd::adfbiodivine::Adf as BdAdf;

use adf_bdd::parser::AdfParser;
use clap::Parser;
use crossbeam_channel::unbounded;
use strum::VariantNames;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct App {
    /// Input filename
    #[clap(parse(from_os_str))]
    input: PathBuf,
    /// Sets the verbosity to 'warn', 'info', 'debug' or 'trace' if -v and -q are not use
    #[clap(long = "rust_log", env)]
    rust_log: Option<String>,
    /// Choose the bdd implementation of either 'biodivine', 'naive', or hybrid
    #[clap(long = "lib", default_value = "hybrid")]
    implementation: String,
    /// Sets log verbosity (multiple times means more verbose)
    #[clap(short, parse(from_occurrences), group = "verbosity")]
    verbose: u8,
    /// Sets log verbosity to only errors
    #[clap(short, group = "verbosity")]
    quiet: bool,
    /// Sorts variables in an lexicographic manner
    #[clap(long = "lx", group = "sorting")]
    sort_lex: bool,
    /// Sorts variables in an alphanumeric manner
    #[clap(long = "an", group = "sorting")]
    sort_alphan: bool,
    /// Compute the grounded model
    #[clap(long = "grd")]
    grounded: bool,
    /// Compute the stable models
    #[clap(long = "stm")]
    stable: bool,
    /// Compute the stable models with the help of modelcounting using heuristics a
    #[clap(long = "stmca")]
    stable_counting_a: bool,
    /// Compute the stable models with the help of modelcounting using heuristics b
    #[clap(long = "stmcb")]
    stable_counting_b: bool,
    /// Compute the stable models with a pre-filter (only hybrid lib-mode)
    #[clap(long = "stmpre")]
    stable_pre: bool,
    /// Compute the stable models with a single-formula rewriting (only hybrid lib-mode)
    #[clap(long = "stmrew")]
    stable_rew: bool,
    /// Compute the stable models with a single-formula rewriting on internal representation(only hybrid lib-mode)
    #[clap(long = "stmrew2")]
    stable_rew2: bool,
    /// Compute the stable models with the nogood-learning based approach
    #[clap(long = "stmng")]
    stable_ng: bool,
    /// Choose which heuristics shall be used by the nogood-learning approach
    #[clap(long, possible_values = adf_bdd::adf::heuristics::Heuristic::VARIANTS.iter().filter(|&v| v != &"Custom"))]
    heu: Option<adf_bdd::adf::heuristics::Heuristic<'static>>,
    /// Compute the two valued models with the nogood-learning based approach
    #[clap(long = "twoval")]
    two_val: bool,
    /// Compute the complete models
    #[clap(long = "com")]
    complete: bool,
    /// Import an adf- bdd state instead of an adf
    #[clap(long)]
    import: bool,
    /// Export the adf-bdd state after parsing and BDD instantiation to the given filename
    #[clap(long)]
    export: Option<PathBuf>,
    /// Set if the (counter-)models shall be computed and printed, possible values are 'nai' and 'mem' for naive and memoization repectively (only works in hybrid and naive mode)
    #[clap(long)]
    counter: Option<String>,
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
        log::info!("Version: {}", clap::crate_version!());
        let input = std::fs::read_to_string(self.input.clone()).expect("Error Reading File");
        match self.implementation.as_str() {
            "hybrid" => {
                let parser = adf_bdd::parser::AdfParser::default();
                match parser.parse()(&input) {
                    Ok(_) => log::info!("[Done] parsing"),
                    Err(e) => {
                        log::error!("Error during parsing:\n{} \n\n cannot continue, panic!", e);
                        panic!("Parsing failed, see log for further details")
                    }
                }
                if self.sort_lex {
                    parser.varsort_lexi();
                }
                if self.sort_alphan {
                    parser.varsort_alphanum();
                }
                let adf = if !self.stable_rew {
                    BdAdf::from_parser(&parser)
                } else {
                    BdAdf::from_parser_with_stm_rewrite(&parser)
                };

                match self.counter.as_deref() {
                    Some("nai") => {
                        let naive_adf = adf.hybrid_step_opt(false);
                        for ac_counts in naive_adf.formulacounts(false) {
                            print!("{:?} ", ac_counts);
                        }
                        println!();
                    }
                    Some("mem") => {
                        let naive_adf = adf.hybrid_step_opt(false);
                        for ac_counts in naive_adf.formulacounts(true) {
                            print!("{:?}", ac_counts);
                        }
                        println!();
                    }
                    Some(_) => {}
                    None => {}
                }
                log::info!("[Start] translate into naive representation");
                let mut naive_adf = adf.hybrid_step();
                log::info!("[Done] translate into naive representation");
                if self.grounded {
                    let grounded = naive_adf.grounded();
                    print!("{}", naive_adf.print_interpretation(&grounded));
                }

                let printer = naive_adf.print_dictionary();

                if self.complete {
                    for model in naive_adf.complete() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }

                if self.two_val {
                    let (sender, receiver) = unbounded();
                    naive_adf.two_val_nogood_channel(self.heu.unwrap_or_default(), sender);
                    for model in receiver.into_iter() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }

                if self.stable {
                    for model in naive_adf.stable() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }

                if self.stable_counting_a {
                    for model in naive_adf.stable_count_optimisation_heu_a() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }

                if self.stable_counting_b {
                    for model in naive_adf.stable_count_optimisation_heu_b() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }

                if self.stable_pre {
                    for model in naive_adf.stable_with_prefilter() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }

                if self.stable_rew || self.stable_rew2 {
                    for model in naive_adf.stable_bdd_representation(&adf) {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }

                if self.stable_ng {
                    for model in naive_adf.stable_nogood(self.heu.unwrap_or_default()) {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }
            }
            "biodivine" => {
                if self.counter.is_some() {
                    log::error!("Modelcounting not supported in biodivine mode");
                }
                let parser = adf_bdd::parser::AdfParser::default();
                match parser.parse()(&input) {
                    Ok(_) => log::info!("[Done] parsing"),
                    Err(e) => {
                        log::error!("Error during parsing:\n{} \n\n cannot continue, panic!", e);
                        panic!("Parsing failed, see log for further details")
                    }
                }
                log::info!("[Done] parsing");
                if self.sort_lex {
                    parser.varsort_lexi();
                }
                if self.sort_alphan {
                    parser.varsort_alphanum();
                }
                let adf = if !self.stable_rew {
                    BdAdf::from_parser(&parser)
                } else {
                    BdAdf::from_parser_with_stm_rewrite(&parser)
                };

                if self.grounded {
                    let grounded = adf.grounded();
                    print!("{}", adf.print_interpretation(&grounded));
                }

                if self.complete {
                    for model in adf.complete() {
                        print!("{}", adf.print_interpretation(&model));
                    }
                }

                if self.stable {
                    for model in adf.stable() {
                        print!("{}", adf.print_interpretation(&model));
                    }
                }

                if self.stable_rew || self.stable_rew2 {
                    for model in adf.stable_bdd_representation() {
                        print!("{}", adf.print_interpretation(&model));
                    }
                }
            }
            _ => {
                let mut adf = if self.import {
                    #[cfg(not(feature = "adhoccounting"))]
                    {
                        serde_json::from_str(&input).expect("Old feature should work")
                    }
                    #[cfg(feature = "adhoccounting")]
                    {
                        let mut result: Adf =
                            serde_json::from_str(&input).expect("Old feature should work");
                        log::debug!("test");
                        result.fix_import();
                        result
                    }
                } else {
                    let parser = AdfParser::default();
                    match parser.parse()(&input) {
                        Ok(_) => log::info!("[Done] parsing"),
                        Err(e) => {
                            log::error!(
                                "Error during parsing:\n{} \n\n cannot continue, panic!",
                                e
                            );
                            panic!("Parsing failed, see log for further details")
                        }
                    }
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

                match self.counter.as_deref() {
                    Some("nai") => {
                        for ac_counts in adf.formulacounts(false) {
                            print!("{:?} ", ac_counts);
                        }
                        println!();
                    }
                    Some("mem") => {
                        for ac_counts in adf.formulacounts(true) {
                            print!("{:?}", ac_counts);
                        }
                        println!();
                    }
                    Some(_) => {}
                    None => {}
                }

                if self.grounded {
                    let grounded = adf.grounded();
                    print!("{}", adf.print_interpretation(&grounded));
                }
                if self.complete {
                    let printer = adf.print_dictionary();
                    for model in adf.complete() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }
                if self.stable {
                    let printer = adf.print_dictionary();
                    for model in adf.stable() {
                        print!("{}", printer.print_interpretation(&model));
                    }
                }
            }
        }
    }
}

fn main() {
    let app = App::parse();
    app.run();
}
