[![Crates.io](https://img.shields.io/crates/v/adf_bdd)](https://crates.io/crates/adf_bdd)
[![docs.rs](https://img.shields.io/docsrs/adf_bdd?label=docs.rs)](https://docs.rs/adf_bdd/latest/adf_bdd/)
![GitHub Workflow Status](https://img.shields.io/github/workflow/status/ellmau/adf-obdd/Code%20coverage%20with%20tarpaulin)
[![Coveralls](https://img.shields.io/coveralls/github/ellmau/adf-obdd)](https://coveralls.io/github/ellmau/adf-obdd)
![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ellmau/adf-obdd?include_prereleases)
![GitHub (Pre-)Release Date](https://img.shields.io/github/release-date-pre/ellmau/adf-obdd?label=release%20from) ![GitHub top language](https://img.shields.io/github/languages/top/ellmau/adf-obdd)
[![GitHub all releases](https://img.shields.io/github/downloads/ellmau/adf-obdd/total)](https://github.com/ellmau/adf-obdd/releases)
![Crates.io](https://img.shields.io/crates/l/adf_bdd)
[![GitHub Discussions](https://img.shields.io/github/discussions/ellmau/adf-obdd)](https://github.com/ellmau/adf-obdd/discussions) ![rust-edition](https://img.shields.io/badge/Rust--edition-2021-blue?logo=rust)

# Abstract Dialectical Frameworks solved by (ordered) Binary Decision Diagrams; developed in Dresden (ADF-oBDD project)

This project is currently split into two parts:
- a [binary (adf-bdd)](adf-bdd.md), which allows one to easily answer semantics questions on abstract dialectical frameworks
- a [library (adf_bdd)](adf_bdd.md), which contains all the necessary algorithms and an open API which compute the answers to the semantics questions


## Abstract Dialectical Frameworks
An abstract dialectical framework (ADF) consists of abstract statements. Each statement has an unique label and might be related to other statements (s) in the ADF. This relation is defined by a so-called acceptance condition (ac), which intuitively is a propositional formula, where the variable symbols are the labels of the statements. An interpretation is a three valued function which maps to each statement a truth value (true, false, undecided). We call such an interpretation a model, if each acceptance condition agrees to the interpration. 
## Ordered Binary Decision Diagram
An ordered binary decision diagram is a normalised representation of binary functions, where satisfiability- and validity checks can be done relatively cheap.

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

# Features

- `adhoccounting` will cache the modelcount on-the-fly during the construction of the BDD
- `adhoccountmodels` allows in addition to compute the models ad-hoc too. Note that the memoization approach for modelcounting does not work correctly if `adhoccounting` is set and `adhoccountmodels` is not.

# Development notes 
Additional information for contribution, testing, and development in general can be found here.
## Contributing to the project
You want to help and contribute to the project? That is great. Please see the [contributing guidelines](https://github.com/ellmau/adf-obdd/blob/main/.github/CONTRIBUTING.md) first.

# Acknowledgements
This work is partly supported by Deutsche Forschungsgemeinschaft (DFG, German Research Foundation) in projects number 389792660 (TRR 248, [Center for Perspicuous Systems](https://www.perspicuous-computing.science/)), 
the Bundesministerium für Bildung und Forschung (BMBF, Federal Ministry of Education and Research) in the
[Center for Scalable Data Analytics and Artificial Intelligence](https://www.scads.de) (ScaDS.AI),
and by the [Center for Advancing Electronics Dresden](https://cfaed.tu-dresden.de) (cfaed).

# Affiliation 
This work has been partly developed by the [Knowledge-Based Systems Group](http://kbs.inf.tu-dresden.de/), [Faculty of Computer Science](https://tu-dresden.de/ing/informatik)  of [TU Dresden](https://tu-dresden.de).

# Disclaimer
Hosting content here does not establish any formal or legal relation to TU Dresden.
