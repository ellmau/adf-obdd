/*!
This module contains all the crate-wide defined heuristic functions.
In addition there is the public enum [Heuristic], which allows to set a heuristic function with the public API.
 */
use super::Adf;
use crate::datatypes::{Term, Var};

use strum::{EnumString, EnumVariantNames};

/// Return value for heuristics.
pub type RetVal = Option<(Var, Term)>;
/// Signature for heuristics functions.
pub type HeuristicFn = dyn Fn(&Adf, &[Term]) -> RetVal;

pub(crate) fn heu_simple(_adf: &Adf, interpr: &[Term]) -> Option<(Var, Term)> {
    for (idx, term) in interpr.iter().enumerate() {
        if !term.is_truth_value() {
            return Some((Var(idx), Term::TOP));
        }
    }
    None
}

pub(crate) fn heu_mc_minpaths_maxvarimp(adf: &Adf, interpr: &[Term]) -> Option<(Var, Term)> {
    interpr
        .iter()
        .enumerate()
        .filter(|(_var, term)| !term.is_truth_value())
        .min_by(|(vara, &terma), (varb, &termb)| {
            match adf
                .bdd
                .paths(terma, true)
                .minimum()
                .cmp(&adf.bdd.paths(termb, true).minimum())
            {
                std::cmp::Ordering::Equal => adf
                    .bdd
                    .passive_var_impact(Var::from(*vara), interpr)
                    .cmp(&adf.bdd.passive_var_impact(Var::from(*varb), interpr)),
                value => value,
            }
        })
        .map(|(var, term)| {
            (
                Var::from(var),
                adf.bdd.paths(*term, true).more_models().into(),
            )
        })
}

pub(crate) fn heu_mc_maxvarimp_minpaths(adf: &Adf, interpr: &[Term]) -> Option<(Var, Term)> {
    interpr
        .iter()
        .enumerate()
        .filter(|(_var, term)| !term.is_truth_value())
        .min_by(|(vara, &terma), (varb, &termb)| {
            match adf
                .bdd
                .passive_var_impact(Var::from(*vara), interpr)
                .cmp(&adf.bdd.passive_var_impact(Var::from(*varb), interpr))
            {
                std::cmp::Ordering::Equal => adf
                    .bdd
                    .paths(terma, true)
                    .minimum()
                    .cmp(&adf.bdd.paths(termb, true).minimum()),

                value => value,
            }
        })
        .map(|(var, term)| {
            (
                Var::from(var),
                adf.bdd.paths(*term, true).more_models().into(),
            )
        })
}

/// Enumeration of all currently implemented heuristics.
/// It represents a public view on the crate-view implementations of heuristics.
#[derive(EnumString, EnumVariantNames, Copy, Clone)]
pub enum Heuristic<'a> {
    /// Implementation of a simple heuristic.
    /// This will just take the first not decided variable and maps it value to (`true`)[Term::TOP].   
    Simple,
    /// Implementation of a heuristic, which which uses minimal number of [paths][crate::obdd::Bdd::paths] and maximal [variable-impact][crate::obdd::Bdd::passive_var_impact to identify the variable to be set.
    /// As the value of the variable value with the maximal model-path is chosen.
    MinModMinPathsMaxVarImp,
    /// Implementation of a heuristic, which which uses maximal [variable-impact][crate::obdd::Bdd::passive_var_impact] and minimal number of [paths][crate::obdd::Bdd::paths] to identify the variable to be set.
    /// As the value of the variable value with the maximal model-path is chosen.
    MinModMaxVarImpMinPaths,
    /// Allow passing in an externally-defined custom heuristic.
    #[strum(disabled)]
    Custom(&'a HeuristicFn),
}

impl Default for Heuristic<'_> {
    fn default() -> Self {
        Self::Simple
    }
}

impl std::fmt::Debug for Heuristic<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Simple => write!(f, "Simple"),
	    Self::MinModMinPathsMaxVarImp => write!(f, "Maximal model-path  count as value and minimum paths with maximal variable impact as variable choice"),
	    Self::MinModMaxVarImpMinPaths => write!(f, "Maximal model-path  count as value and maximal variable impact with minimum paths as variable choice"),
            Self::Custom(_) => f.debug_tuple("Custom function").finish(),
        }
    }
}

impl Heuristic<'_> {
    pub(crate) fn get_heuristic(&self) -> &(dyn Fn(&Adf, &[Term]) -> RetVal + '_) {
        match self {
            Heuristic::Simple => &heu_simple,
            Heuristic::MinModMinPathsMaxVarImp => &heu_mc_minpaths_maxvarimp,
            Heuristic::MinModMaxVarImpMinPaths => &heu_mc_maxvarimp_minpaths,
            Self::Custom(f) => f,
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::datatypes::Term;
    use crate::datatypes::Var;

    #[test]
    fn debug_out() {
        dbg!(Heuristic::Simple);
        dbg!(Heuristic::MinModMaxVarImpMinPaths);
        dbg!(Heuristic::MinModMinPathsMaxVarImp);
        dbg!(Heuristic::Custom(&|_adf: &Adf,
                                 _int: &[Term]|
         -> Option<(Var, Term)> { None }));
    }
}
