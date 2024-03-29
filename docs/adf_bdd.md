[![Crates.io](https://img.shields.io/crates/v/adf_bdd)](https://crates.io/crates/adf_bdd)
[![docs.rs](https://img.shields.io/docsrs/adf_bdd?label=docs.rs)](https://docs.rs/adf_bdd/latest/adf_bdd/)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ellmau/adf-obdd/Code%20coverage%20with%20tarpaulin)
[![Coveralls](https://img.shields.io/coveralls/github/ellmau/adf-obdd)](https://coveralls.io/github/ellmau/adf-obdd)
![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ellmau/adf-obdd?include_prereleases)
![GitHub (Pre-)Release Date](https://img.shields.io/github/release-date-pre/ellmau/adf-obdd?label=release%20from) ![GitHub top language](https://img.shields.io/github/languages/top/ellmau/adf-obdd)
[![GitHub all releases](https://img.shields.io/github/downloads/ellmau/adf-obdd/total)](https://github.com/ellmau/adf-obdd/releases)
![Crates.io](https://img.shields.io/crates/l/adf_bdd)
[![GitHub Discussions](https://img.shields.io/github/discussions/ellmau/adf-obdd)](https://github.com/ellmau/adf-obdd/discussions) ![rust-edition](https://img.shields.io/badge/Rust--edition-2021-blue?logo=rust)

| [Home](index.md) | [Binary](adf-bdd.md) | [Library](adf_bdd.md)| [Web-Service](https://adf-bdd.dev) | [Repository](https://github.com/ellmau/adf-obdd) |
|--- | --- | --- | --- | --- |

# Abstract Dialectical Frameworks solved by Binary Decision Diagrams; developed in Dresden (ADF_BDD) 
This library contains an efficient representation of Abstract Dialectical Frameworks (ADf) by utilising an implementation of Ordered Binary Decision Diagrams (OBDD)

## Noteworthy relations between ADF semantics

They can be easily identified though:

* The computation is always in the same order
    * grd
    * com
    * stm
* We know that there is always exactly one grounded model
* We know that there always exist at least one complete model (i.e. the grounded one)
* We know that there does not need to exist a stable model
* We know that every stable model is a complete model too


## Ordered Binary Decision Diagram

An ordered binary decision diagram is a normalised representation of binary functions, where satisfiability- and validity checks can be done relatively cheap.

Note that one advantage of this implementation is that only one oBDD is used for all acceptance conditions. This can be done because all of them have the identical signature (i.e. the set of all statements + top and bottom concepts). Due to this uniform representation reductions on subformulae which are shared by two or more statements only need to be computed once and is already cached in the data structure for further applications.

The used algorithm to create a BDD, based on a given formula does not perform well on bigger formulae, therefore it is possible to use a state-of-the art library to instantiate the BDD (https://github.com/sybila/biodivine-lib-bdd). It is possible to either stay with the biodivine library or switch back to the variant implemented by adf-bdd. The variant implemented in this library offers reuse of already done reductions and memoisation techniques, which are not offered by biodivine. In addition some further features, like counter-model counting is not supported by biodivine.

Note that import and export only works if the naive library is chosen

## Input-file format:

Each statement is defined by an ASP-style unary predicate s, where the enclosed term represents the label of the statement. The binary predicate ac relates each statement to one propositional formula in prefix notation, with the logical operations and constants as follows:
```plain
and(x,y): conjunction
or(x,y): disjunctin
iff(x,Y): if and only if
xor(x,y): exclusive or
neg(x): classical negation
c(v): constant symbol “verum” - tautology/top
c(f): constant symbol “falsum” - inconsistency/bot
```

### Example input file:
```plain
s(a).
s(b).
s(c).
s(d).

ac(a,c(v)).
ac(b,or(a,b)).
ac(c,neg(b)).
ac(d,d).
```

## Usage examples

First parse a given ADF and sort the statements, if needed.

```rust
use adf_bdd::parser::AdfParser;
use adf_bdd::adf::Adf;
// use the above example as input
let input = "s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,or(a,b)).ac(c,neg(b)).ac(d,d).";
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
// sort lexicographic
parser.varsort_lexi();
```
use the naive/in-crate implementation

```rust
// create Adf
let mut adf = Adf::from_parser(&parser);
// compute and print the complete models
let printer = adf.print_dictionary();
for model in adf.complete() {
    print!("{}", printer.print_interpretation(&model));
}
```
use the biodivine implementation
```rust
// create Adf
let adf = adf_bdd::adfbiodivine::Adf::from_parser(&parser);
// compute and print the complete models
let printer = adf.print_dictionary();
for model in adf.complete() {
    print!("{}", printer.print_interpretation(&model));
}
```
use the hybrid approach implementation
```rust
// create biodivine Adf
let badf = adf_bdd::adfbiodivine::Adf::from_parser(&parser);
// instantiate the internally used adf after the reduction done by biodivine
let mut adf = badf.hybrid_step();
// compute and print the complete models
let printer = adf.print_dictionary();
for model in adf.complete() {
    print!("{}", printer.print_interpretation(&model));
}
```

use the new `NoGood`-based algorithm and utilise the new interface with channels:
```rust
use adf_bdd::parser::AdfParser;
use adf_bdd::adf::Adf;
use adf_bdd::adf::heuristics::Heuristic;
use adf_bdd::datatypes::{Term, adf::VarContainer};
// create a channel
let (s, r) = crossbeam_channel::unbounded();
let variables = VarContainer::default();
let variables_worker = variables.clone();
// spawn a solver thread
let solving = std::thread::spawn(move || {
   // use the above example as input
   let input = "s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,or(a,b)).ac(c,neg(b)).ac(d,d).";
   let parser = AdfParser::with_var_container(variables_worker);
   parser.parse()(&input).expect("parsing worked well");
   // use hybrid approach
   let mut adf = adf_bdd::adfbiodivine::Adf::from_parser(&parser).hybrid_step();
   // compute stable with the simple heuristic
   adf.stable_nogood_channel(Heuristic::Simple, s);
});

let printer = variables.print_dictionary();
// print results as they are computed
while let Ok(result) = r.recv() {
   print!("stable model: {:?} \n", result);
   // use dictionary
   print!("stable model with variable names: {}", printer.print_interpretation(&result));
}
// waiting for the other thread to close
solving.join().unwrap();
```

# Acknowledgements
This work is partly supported by Deutsche Forschungsgemeinschaft (DFG, German Research Foundation) in projects number 389792660 (TRR 248, [Center for Perspicuous Systems](https://www.perspicuous-computing.science/)), 
the Bundesministerium für Bildung und Forschung (BMBF, Federal Ministry of Education and Research) in the
[Center for Scalable Data Analytics and Artificial Intelligence](https://www.scads.de) (ScaDS.AI),
and by the [Center for Advancing Electronics Dresden](https://cfaed.tu-dresden.de) (cfaed).

# Affiliation 
This work has been partly developed by the [Knowledge-Based Systems Group](http://kbs.inf.tu-dresden.de/), [Faculty of Computer Science](https://tu-dresden.de/ing/informatik)  of [TU Dresden](https://tu-dresden.de).

# Disclaimer
Hosting content here does not establish any formal or legal relation to TU Dresden.
