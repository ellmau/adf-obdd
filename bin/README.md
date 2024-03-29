[![Crates.io](https://img.shields.io/crates/v/adf-bdd-bin)](https://crates.io/crates/adf-bdd-bin)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/ellmau/adf-obdd/codecov.yml?branch=main)
[![Coveralls](https://img.shields.io/coveralls/github/ellmau/adf-obdd)](https://coveralls.io/github/ellmau/adf-obdd) 
![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ellmau/adf-obdd?include_prereleases) 
![GitHub (Pre-)Release Date](https://img.shields.io/github/release-date-pre/ellmau/adf-obdd?label=release%20from) 
![GitHub top language](https://img.shields.io/github/languages/top/ellmau/adf-obdd) 
[![GitHub all releases](https://img.shields.io/github/downloads/ellmau/adf-obdd/total)](https://github.com/ellmau/adf-obdd/releases) 
[![GitHub Discussions](https://img.shields.io/github/discussions/ellmau/adf-obdd)](https://github.com/ellmau/adf-obdd/discussions) 
![rust-edition](https://img.shields.io/badge/Rust--edition-2021-blue?logo=rust)

# Abstract Dialectical Frameworks solved by Binary Decision Diagrams; developed in Dresden (ADF-BDD) 
This is the readme for the executable solver.

## Abstract Dialectical Frameworks
An abstract dialectical framework (ADF) consists of abstract statements. Each statement has an unique label and might be related to other statements (s) in the ADF. This relation is defined by a so-called acceptance condition (ac), which intuitively is a propositional formula, where the variable symbols are the labels of the statements. An interpretation is a three valued function which maps to each statement a truth value (true, false, undecided). We call such an interpretation a model, if each acceptance condition agrees to the interpration. 
## Ordered Binary Decision Diagram
An ordered binary decision diagram is a normalised representation of binary functions, where satisfiability- and validity checks can be done relatively cheap.

## Usage
```
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

Note that import and export only works if the naive library is chosen

Right now there is no additional information to the computed models, so if you use --com --grd --stm the borders between the results are not obviously communicated.
They can be easily identified though:
- The computation is always in the same order
  - grd
  - com
  - stm
- We know that there is always exactly one grounded model
- We know that there always exist at least one complete model (i.e. the grounded one)
- We know that there does not need to exist a stable model
- We know that every stable model is a complete model too


## Input-file format:
Each statement is defined by an ASP-style unary predicate s, where the enclosed term represents the label of the statement.
The binary predicate ac relates each statement to one propositional formula in prefix notation, with the logical operations and constants as follows:
- and(x,y): conjunction
- or(x,y): disjunctin
- iff(x,Y): if and only if
- xor(x,y): exclusive or
- neg(x): classical negation
- c(v): constant symbol "verum" - tautology/top
- c(f): constant symbol "falsum" - inconsistency/bot

# Development notes
To build the binary, you need to run
```bash
$> cargo build --workspace --release
```

To build the binary with debug-symbols, run
```bash
$> cargo build --workspace
```

To run all the tests placed in the submodule you need to run
```bash
$> git submodule init
```
at the first time.
Afterwards you need to update the content of the submodule to be on the currently used revision by
```bash
$> git submodule update
```

The tests can be started by using the test-framework of cargo, i.e.
```bash
$> cargo test
```
Note that some of the instances are quite big and it might take some time to finish all the tests.
If you do not initialise the submodule, tests will "only" run on the other unit-tests and (possibly forthcoming) other integration tests.
Due to the way of the generated test-modules you need to call 
```bash
$> cargo clean
```
if you change some of your test-cases.

To remove the tests just type
```bash
$> git submodule deinit res/adf-instances
```
or
```bash
$> git submodule deinit --all
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
