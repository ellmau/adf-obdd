/*!
This library contains an efficient representation of `Abstract Dialectical Frameworks (ADF)` by utilising an implementation of `Ordered Binary Decision Diagrams (OBDD)`

# Abstract Dialectical Frameworks
An `abstract dialectical framework` consists of abstract statements. Each statement has a unique label and might be related to other statements (s) in the ADF. This relation is defined by a so-called acceptance condition (ac), which intuitively is a propositional formula, where the variable symbols are the labels of the statements. An interpretation is a three valued function which maps to each statement a truth value (true, false, undecided). We call such an interpretation a model, if each acceptance condition agrees to the interpretation.

## Noteworthy relations between semantics

- The computation is always in the same order
  - grd
  - com
  - stm
- We know that there is always exactly one grounded model
- We know that there always exists at least one complete model (i.e., the grounded one)
- We know that there does not need to exist a stable model
- We know that every stable model is a complete model too

# Reduced Ordered Binary Decision Diagram (roBDD)
A `reduced ordered binary decision diagram` is a normalised representation of binary functions, where satisfiability- and validity checks can be done relatively cheap and no redundant information is stored.

Note that one advantage of this implementation is that only one structure is used for all acceptance conditions. This can be done because all of them have the identical signature (i.e., the set of all statements + top and bottom concepts).
Due to this uniform representation reductions on subformulae which are shared by two or more statements only need to be computed once and will be cached in the data structure for further applications.

The naively used algorithm to create an roBDD, based on a given formula does not perform well on bigger formulae, therefore it is possible to use a state-of-the art library to instantiate the roBDD (<https://github.com/sybila/biodivine-lib-bdd>).
It is possible to either stay with the biodivine library or switch back to the variant implemented by adf-bdd.
The variant implemented in this library offers reuse of already done reductions and memoisation techniques, which are not offered by biodivine.
In addition some further features, like counter-model counting is not supported by biodivine.

Note that import and export only works if the naive library is chosen.

# Input-file format
Each statement is defined by an ASP-style unary predicate `s`, where the enclosed term represents the label of the statement.
The binary predicate `ac` relates each statement to one propositional formula in prefix notation, with the logical operations and constants as follows:
- `and(x,y)`: conjunction
- `or(x,y)`: disjunction
- `iff(x,Y)`: if and only if
- `xor(x,y)`: exclusive or
- `neg(x)`: classical negation
- `c(v)`: constant symbol "verum" - tautology/top
- `c(f)`: constant symbol "falsum" - inconsistency/bot
*/

/*!
## Example input file:
```prolog
s(a).
s(b).
s(c).
s(d).

ac(a,c(v)).
ac(b,or(a,b)).
ac(c,neg(b)).
ac(d,d).
```
*/

/*!
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
### use the naive/in-crate implementation
```rust
# use adf_bdd::parser::AdfParser;
# use adf_bdd::adf::Adf;
# // use the above example as input
# let input = "s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,or(a,b)).ac(c,neg(b)).ac(d,d).";
# let parser = AdfParser::default();
# match parser.parse()(&input) {
#    Ok(_) => log::info!("[Done] parsing"),
#    Err(e) => {
#    log::error!(
#        "Error during parsing:\n{} \n\n cannot continue, panic!",
#        e
#        );
#        panic!("Parsing failed, see log for further details")
#    }
# }
# // sort lexicographic
# parser.varsort_lexi();
// create Adf
let mut adf = Adf::from_parser(&parser);
// compute and print the complete models
let printer = adf.print_dictionary();
for model in adf.complete() {
    print!("{}", printer.print_interpretation(&model));
}
```
### use the biodivine implementation
```rust
# use adf_bdd::parser::AdfParser;
# use adf_bdd::adf::Adf;
# // use the above example as input
# let input = "s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,or(a,b)).ac(c,neg(b)).ac(d,d).";
# let parser = AdfParser::default();
# match parser.parse()(&input) {
#    Ok(_) => log::info!("[Done] parsing"),
#    Err(e) => {
#    log::error!(
#        "Error during parsing:\n{} \n\n cannot continue, panic!",
#        e
#        );
#        panic!("Parsing failed, see log for further details")
#    }
# }
# // sort lexicographic
# parser.varsort_lexi();
// create Adf
let adf = adf_bdd::adfbiodivine::Adf::from_parser(&parser);
// compute and print the complete models
let printer = adf.print_dictionary();
for model in adf.complete() {
    print!("{}", printer.print_interpretation(&model));
}
```
### use the hybrid approach implementation
```rust
# use adf_bdd::parser::AdfParser;
# use adf_bdd::adf::Adf;
# // use the above example as input
# let input = "s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,or(a,b)).ac(c,neg(b)).ac(d,d).";
# let parser = AdfParser::default();
# match parser.parse()(&input) {
#    Ok(_) => log::info!("[Done] parsing"),
#    Err(e) => {
#    log::error!(
#        "Error during parsing:\n{} \n\n cannot continue, panic!",
#        e
#        );
#        panic!("Parsing failed, see log for further details")
#    }
# }
# // sort lexicographic
# parser.varsort_lexi();
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

### Using the [`NoGood`]-learner approach, together with the [`crossbeam-channel`] implementation
This can be used to have a worker and a consumer thread to print the results as they are computed.
Please note that the [`NoGood`]-learner needs a heuristics function to work.
The enum [`Heuristic`][adf_bdd::adf::heuristics::Heuristic] allows one to choose a pre-defined heuristic, or implement a `Custom` one.
```rust
use adf_bdd::parser::AdfParser;
use adf_bdd::adf::Adf;
use adf_bdd::adf::heuristics::Heuristic;
use adf_bdd::datatypes::Term;
// create a channel
let (s, r) = crossbeam_channel::unbounded();
// spawn a solver thread
let solving = std::thread::spawn(move || {
   // use the above example as input
   let input = "s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,or(a,b)).ac(c,neg(b)).ac(d,d).";
   let parser = AdfParser::default();
   parser.parse()(&input).expect("parsing worked well");
   // use hybrid approach
   let mut adf = adf_bdd::adfbiodivine::Adf::from_parser(&parser).hybrid_step();
   // compute stable with the simple heuristic
   adf.stable_nogood_channel(Heuristic::Simple, s);
});

// print results as they are computed
while let Ok(result) = r.recv() {
   println!("stable model: {:?}", result);
#  assert_eq!(result, vec![Term(1),Term(1),Term(0),Term(0)]);
}
// waiting for the other thread to close
solving.join().unwrap();

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

pub mod adf;
pub mod adfbiodivine;
pub mod datatypes;
pub mod nogoods;
pub mod obdd;
pub mod parser;
#[cfg(test)]
mod test;
