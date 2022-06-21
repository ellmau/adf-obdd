/*!
This module contains all the crate-wide defined heuristic functions.
In addition there is the public enum [Heuristic], which allows to set a heuristic function with the public API.
 */
use super::Adf;
use crate::datatypes::{Term, Var};

pub(crate) fn heu_simple(_adf: &Adf, interpr: &[Term]) -> Option<(Var, Term)> {
    for (idx, term) in interpr.iter().enumerate() {
        if !term.is_truth_value() {
            return Some((Var(idx), Term::TOP));
        }
    }
    None
}

type RetVal = Option<(Var, Term)>;

/// Enumeration of all currently implemented heuristics.
/// It represents a public view on the crate-view implementations of heuristics.
#[derive(Debug, Clone, Copy)]
pub enum Heuristic {
    /// Implementation of a simple heuristic.
    /// This will just take the first not decided variable and maps it value to (`true`)[Term::TOP].   
    Simple,
}

impl Heuristic {
    pub(crate) fn get_heuristic(&self) -> Box<dyn Fn(&Adf, &[Term]) -> RetVal> {
        match self {
            Heuristic::Simple => Box::new(heu_simple),
        }
    }
}
