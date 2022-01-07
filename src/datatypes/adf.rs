//! Repesentation of all needed ADF based datatypes

use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use super::{Term, Var};

pub(crate) struct VarContainer {
    names: Rc<RefCell<Vec<String>>>,
    mapping: Rc<RefCell<HashMap<String, usize>>>,
}

impl Default for VarContainer {
    fn default() -> Self {
        VarContainer {
            names: Rc::new(RefCell::new(Vec::new())),
            mapping: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl VarContainer {
    pub fn from_parser(
        names: Rc<RefCell<Vec<String>>>,
        mapping: Rc<RefCell<HashMap<String, usize>>>,
    ) -> VarContainer {
        VarContainer { names, mapping }
    }

    pub fn variable(&self, name: &str) -> Option<Var> {
        self.mapping.borrow().get(name).map(|val| Var(*val))
    }

    pub fn name(&self, var: Var) -> Option<String> {
        self.names.borrow().get(var.value()).cloned()
    }

    pub fn names(&self) -> Rc<RefCell<Vec<String>>> {
        Rc::clone(&self.names)
    }
}

/// A struct to print a representation, it will be instantiated by [Adf] by calling the method [`Adf::print_interpretation`].
pub struct PrintableInterpretation<'a> {
    interpretation: &'a [Term],
    ordering: &'a VarContainer,
}

impl<'a> PrintableInterpretation<'a> {
    pub(crate) fn new(interpretation: &'a [Term], ordering: &'a VarContainer) -> Self {
        Self {
            interpretation,
            ordering,
        }
    }
}

impl Display for PrintableInterpretation<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.interpretation
            .iter()
            .enumerate()
            .for_each(|(pos, term)| {
                if term.is_truth_value() {
                    if term.is_true() {
                        write!(f, "T(").expect("writing Interpretation failed!");
                    } else {
                        write!(f, "F(").expect("writing Interpretation failed!");
                    }
                } else {
                    write!(f, "u(").expect("writing Interpretation failed!");
                }
                write!(f, "{}) ", self.ordering.name(Var(pos)).unwrap())
                    .expect("writing Interpretation failed!");
            });
        writeln!(f)
    }
}

/// Provides an Iterator, which contains all two valued Interpretations, with respect to the given
/// 3-valued interpretation.

#[derive(Debug)]
pub struct TwoValuedInterpretationsIterator {
    indexes: Vec<usize>,
    current: Option<Vec<Term>>,
    started: bool,
}

impl TwoValuedInterpretationsIterator {
    /// Creates a new iterable structure, which represents all two-valued interpretations wrt. the given 3-valued interpretation
    pub fn new(term: &[Term]) -> Self {
        let indexes = term
            .iter()
            .enumerate()
            .filter_map(|(idx, &v)| (!v.is_truth_value()).then(|| idx))
            .rev()
            .collect::<Vec<_>>();
        let current = term
            .iter()
            .map(|&v| if !v.is_truth_value() { Term::BOT } else { v })
            .collect::<Vec<_>>();

        Self {
            indexes,
            started: false,
            current: Some(current),
        }
    }
}

impl Iterator for TwoValuedInterpretationsIterator {
    type Item = Vec<Term>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            if let Some(current) = &self.current {
                if let Some((idx, &at)) = self
                    .indexes
                    .iter()
                    .enumerate()
                    .find(|(_, &idx)| current[idx] == Term::BOT)
                {
                    let mut result = current.clone();
                    result[at] = Term::TOP;
                    for &at in self.indexes[0..idx].iter() {
                        result[at] = Term::BOT;
                    }
                    log::trace!("{:?} -> {:?}", current, result);
                    self.current = Some(result);
                } else {
                    self.current = None;
                }
            };
        } else {
            self.started = true;
        }

        self.current.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn init_varcontainer() {
        let vc = VarContainer::default();
        assert_eq!(vc.variable("foo"), None);
    }

    #[test]
    fn two_valued_interpretations() {
        let testinterpretation = vec![Term::TOP, Term(22), Term::BOT, Term(12), Term::TOP];
        let mut iter = TwoValuedInterpretationsIterator::new(&testinterpretation);
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::BOT, Term::BOT, Term::BOT, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::BOT, Term::BOT, Term::TOP, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::TOP, Term::BOT, Term::BOT, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::TOP, Term::BOT, Term::TOP, Term::TOP])
        );
        assert_eq!(iter.next(), None);

        let testinterpretation = vec![Term(22), Term(12)];
        let mut iter = TwoValuedInterpretationsIterator::new(&testinterpretation);
        assert_eq!(iter.next(), Some(vec![Term::BOT, Term::BOT]));
        assert_eq!(iter.next(), Some(vec![Term::BOT, Term::TOP]));
        assert_eq!(iter.next(), Some(vec![Term::TOP, Term::BOT]));
        assert_eq!(iter.next(), Some(vec![Term::TOP, Term::TOP]));
        assert_eq!(iter.next(), None);
    }
}
