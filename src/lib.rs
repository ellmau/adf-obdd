//! This library contains an efficient representation of `Abstract Dialectical Frameworks (ADf)` by utilising an implementation of `Ordered Binary Decision Diagrams (OBDD)`
//!
//! # Abstract Dialectical Frameworks
//! An `abstract dialectical framework` consists of abstract statements. Each statement has an unique label and might be related to other statements (s) in the ADF. This relation is defined by a so-called acceptance condition (ac), which intuitively is a propositional formula, where the variable symbols are the labels of the statements. An interpretation is a three valued function which maps to each statement a truth value (true, false, undecided). We call such an interpretation a model, if each acceptance condition agrees to the interpration.
//! # Ordered Binary Decision Diagram
//! An `ordered binary decision diagram` is a normalised representation of binary functions, where satisfiability- and validity checks can be done relatively cheap.
//!
//! Note that one advantage of this implementation is that only one oBDD is used for all acceptance conditions. This can be done because all of them have the identical signature (i.e. the set of all statements + top and bottom concepts).
//! Due to this uniform representation reductions on subformulae which are shared by two or more statements only need to be computed once and is already cached in the data structure for further applications.
//!
//! # Usage
//! ```plain
//! USAGE:
//!     adf_bdd [FLAGS] [OPTIONS] <input>
//!
//! FLAGS:
//!         --com        Compute the complete models
//!         --grd        Compute the grounded model
//!     -h, --help       Prints help information
//!         --import     Import an adf- bdd state instead of an adf
//!     -q               Sets log verbosity to only errors
//!         --an         Sorts variables in an alphanumeric manner
//!         --lx         Sorts variables in an lexicographic manner
//!         --stm        Compute the stable models
//!     -V, --version    Prints version information
//!     -v               Sets log verbosity (multiple times means more verbose)
//!
//! OPTIONS:
//!         --export <export>         Export the adf-bdd state after parsing and BDD instantiation to the given filename
//!         --lib <implementation>    choose the bdd implementation of either 'biodivine', 'naive', or hybrid [default:
//!                                   biodivine]
//!         --rust_log <rust-log>     Sets the verbosity to 'warn', 'info', 'debug' or 'trace' if -v and -q are not use
//!                                   [env: RUST_LOG=debug]
//!
//! ARGS:
//!     <input>    Input filename
//! ```
//!
//! Note that import and export only works if the naive library is chosen
//!
//! Right now there is no additional information to the computed models, so if you use --com --grd --stm the borders between the results are not obviously communicated.
//! They can be easily identified though:
//! - The computation is always in the same order
//!   - grd
//!   - com
//!   - stm
//! - We know that there is always exactly one grounded model
//! - We know that there always exist at least one complete model (i.e. the grounded one)
//! - We know that there does not need to exist a stable model
//! - We know that every stable model is a complete model too
//!
//! # Input-file format:
//! Each statement is defined by an ASP-style unary predicate s, where the enclosed term represents the label of the statement.
//! The binary predicate ac relates each statement to one propositional formula in prefix notation, with the logical operations and constants as follows:
//! - and(x,y): conjunction
//! - or(x,y): disjunctin
//! - iff(x,Y): if and only if
//! - xor(x,y): exclusive or
//! - neg(x): classical negation
//! - c(v): constant symbol "verum" - tautology/top
//! - c(f): constant symbol "falsum" - inconsistency/bot

//! ## Example input file:
//! ```prolog
//! s(a).
//! s(b).
//! s(c).
//! s(d).
//!
//! ac(a,c(v)).
//! ac(b,or(a,b)).
//! ac(c,neg(b)).
//! ac(d,d).
//! ```
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
pub mod obdd;
pub mod parser;
#[cfg(test)]
mod test;
//pub mod obdd2;
