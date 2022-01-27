![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ellmau/adf-obdd/Code%20coverage%20with%20tarpaulin) [![Coveralls](https://img.shields.io/coveralls/github/ellmau/adf-obdd)](https://coveralls.io/github/ellmau/adf-obdd) ![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ellmau/adf-obdd?include_prereleases) ![GitHub (Pre-)Release Date](https://img.shields.io/github/release-date-pre/ellmau/adf-obdd?label=release%20from) ![GitHub top language](https://img.shields.io/github/languages/top/ellmau/adf-obdd) ![GitHub all releases](https://img.shields.io/github/downloads/ellmau/adf-obdd/total) ![GitHub Discussions](https://img.shields.io/github/discussions/ellmau/adf-obdd)
# Solver for ADFs grounded semantics by utilising OBDDs - ordered binary decision diagrams


## Abstract Dialectical Frameworks
An abstract dialectical framework (ADF) consists of abstract statements. Each statement has an unique label and might be related to other statements (s) in the ADF. This relation is defined by a so-called acceptance condition (ac), which intuitively is a propositional formula, where the variable symbols are the labels of the statements. An interpretation is a three valued function which maps to each statement a truth value (true, false, undecided). We call such an interpretation a model, if each acceptance condition agrees to the interpration. 
## Ordered Binary Decision Diagram
An ordered binary decision diagram is a normalised representation of binary functions, where satisfiability- and validity checks can be done relatively cheap.

## Usage
```
USAGE:
    adf_bdd [FLAGS] [OPTIONS] <input>

FLAGS:
        --com        Compute the complete models
        --grd        Compute the grounded model
    -h, --help       Prints help information
        --import     Import an adf- bdd state instead of an adf
    -q               Sets log verbosity to only errors
        --an         Sorts variables in an alphanumeric manner
        --lx         Sorts variables in an lexicographic manner
        --stm        Compute the stable models
    -V, --version    Prints version information
    -v               Sets log verbosity (multiple times means more verbose)

OPTIONS:
        --export <export>         Export the adf-bdd state after parsing and BDD instantiation to the given filename
        --lib <implementation>    choose the bdd implementation of either 'biodivine', 'naive', or hybrid [default:
                                  biodivine]
        --rust_log <rust-log>     Sets the verbosity to 'warn', 'info', 'debug' or 'trace' if -v and -q are not use
                                  [env: RUST_LOG=debug]

ARGS:
    <input>    Input filename
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
