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

/// Return value for heuristics.
pub type RetVal = Option<(Var, Term)>;
/// Signature for heuristics functions.
pub type HeuristicFn = dyn Fn(&Adf, &[Term]) -> RetVal;

/// Enumeration of all currently implemented heuristics.
/// It represents a public view on the crate-view implementations of heuristics.
pub enum Heuristic<'a> {
    /// Implementation of a simple heuristic.
    /// This will just take the first not decided variable and maps it value to (`true`)[Term::TOP].   
    Simple,
    /// Allow passing in an externally-defined custom heuristic.
    Custom(&'a HeuristicFn),
}

impl std::fmt::Debug for Heuristic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "Simple"),
            Self::Custom(_) => f.debug_tuple("Custom function").finish(),
        }
    }
}

impl Heuristic<'_> {
    pub(crate) fn get_heuristic(&self) -> &(dyn Fn(&Adf, &[Term]) -> RetVal + '_) {
        match self {
            Heuristic::Simple => &heu_simple,
            Self::Custom(f) => f,
        }
    }
}
