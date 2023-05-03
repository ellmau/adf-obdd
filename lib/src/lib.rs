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

### Using the [`NoGood`][crate::nogoods::NoGood]-learner approach, together with the [`crossbeam-channel`] implementation
This can be used to have a worker and a consumer thread to print the results as they are computed.
Please note that the [`NoGood`][crate::nogoods::NoGood]-learner needs a heuristics function to work.
The enum [`Heuristic`][crate::adf::heuristics::Heuristic] allows one to choose a pre-defined heuristic, or implement a `Custom` one.
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
#  assert_eq!(result, vec![Term(1),Term(1),Term(0),Term(0)]);
}
// waiting for the other thread to close
solving.join().unwrap();

```

### Serialize and Deserialize custom datastructures representing an [`adf::Adf`]

The Web Application <https://adf-bdd.dev> uses custom datastructures that are stored in a mongodb which inspired this example.

```rust
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use adf_bdd::datatypes::adf::VarContainer;
use adf_bdd::datatypes::{BddNode, Term, Var};
use adf_bdd::obdd::Bdd;
use adf_bdd::parser::AdfParser;
use adf_bdd::adf::Adf;

// Custom Datastructures for (De-)Serialization

# #[derive(PartialEq, Debug)]
#[derive(Deserialize, Serialize)]
struct MyCustomVarContainer {
    names: Vec<String>,
    mapping: HashMap<String, String>,
}

impl From<VarContainer> for MyCustomVarContainer {
    fn from(source: VarContainer) -> Self {
        Self {
            names: source.names().read().unwrap().clone(),
            mapping: source
                .mappings()
                .read()
                .unwrap()
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
        }
    }
}

impl From<MyCustomVarContainer> for VarContainer {
    fn from(source: MyCustomVarContainer) -> Self {
        Self::from_parser(
            Arc::new(RwLock::new(source.names)),
            Arc::new(RwLock::new(
                source
                    .mapping
                    .into_iter()
                    .map(|(k, v)| (k, v.parse().unwrap()))
                    .collect(),
            )),
        )
    }
}

# #[derive(PartialEq, Debug)]
#[derive(Deserialize, Serialize)]
struct MyCustomBddNode {
    var: String,
    lo: String,
    hi: String,
}

impl From<BddNode> for MyCustomBddNode {
    fn from(source: BddNode) -> Self {
        Self {
            var: source.var().0.to_string(),
            lo: source.lo().0.to_string(),
            hi: source.hi().0.to_string(),
        }
    }
}

impl From<MyCustomBddNode> for BddNode {
    fn from(source: MyCustomBddNode) -> Self {
        Self::new(
            Var(source.var.parse().unwrap()),
            Term(source.lo.parse().unwrap()),
            Term(source.hi.parse().unwrap()),
        )
    }
}

# #[derive(PartialEq, Debug)]
#[derive(Deserialize, Serialize)]
struct MyCustomAdf {
    ordering: MyCustomVarContainer,
    bdd: Vec<MyCustomBddNode>,
    ac: Vec<String>,
}

impl From<Adf> for MyCustomAdf {
    fn from(source: Adf) -> Self {
        Self {
            ordering: source.ordering.into(),
            bdd: source.bdd.nodes.into_iter().map(Into::into).collect(),
            ac: source.ac.into_iter().map(|t| t.0.to_string()).collect(),
        }
    }
}

impl From<MyCustomAdf> for Adf {
    fn from(source: MyCustomAdf) -> Self {
        let bdd = Bdd::from(source.bdd.into_iter().map(Into::into).collect::<Vec<BddNode>>());

        Adf::from((
            source.ordering.into(),
            bdd,
            source
                .ac
                .into_iter()
                .map(|t| Term(t.parse().unwrap()))
                .collect(),
        ))
    }
}

// use the above example as input
let input = "s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,or(a,b)).ac(c,neg(b)).ac(d,d).";
let parser = AdfParser::default();
parser.parse()(&input).unwrap();

// create Adf
let adf = Adf::from_parser(&parser);

// cast into custom struct
let my_custom_adf: MyCustomAdf = adf.into();

// stringify to json
let json: String = serde_json::to_string(&my_custom_adf).unwrap();

// parse json
let parsed_custom_adf: MyCustomAdf = serde_json::from_str(&json).unwrap();

// cast into lib struct that resembles the original Adf
let parsed_adf: Adf = parsed_custom_adf.into();

# let my_custom_adf2: MyCustomAdf = parsed_adf.into();
# assert_eq!(my_custom_adf, my_custom_adf2);
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
