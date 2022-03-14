//! Repesentation of all needed ADF based datatypes

use super::{Term, Var};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

#[derive(Serialize, Deserialize, Debug)]
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

    pub fn copy(from: &Self) -> Self {
        VarContainer {
            names: from.names.clone(),
            mapping: from.mapping.clone(),
        }
    }

    pub fn variable(&self, name: &str) -> Option<Var> {
        self.mapping.borrow().get(name).map(|val| Var(*val))
    }

    pub fn name(&self, var: Var) -> Option<String> {
        self.names.borrow().get(var.value()).cloned()
    }

    #[allow(dead_code)]
    pub fn names(&self) -> Rc<RefCell<Vec<String>>> {
        Rc::clone(&self.names)
    }
}
/// A struct which holds the dictionary to print interpretations and allows to instantiate printable interpretations
#[derive(Debug)]
pub struct PrintDictionary {
    ordering: VarContainer,
}

impl PrintDictionary {
    pub(crate) fn new(order: &VarContainer) -> Self {
        Self {
            ordering: VarContainer::copy(order),
        }
    }
    /// creates a [PrintableInterpretation] for output purposes
    pub fn print_interpretation<'a, 'b>(
        &'a self,
        interpretation: &'b [Term],
    ) -> PrintableInterpretation<'b>
    where
        'a: 'b,
    {
        PrintableInterpretation::new(interpretation, &self.ordering)
    }
}

/// A struct to print a representation, it will be instantiated by [Adf][crate::adf::Adf] by calling the method [print_interpretation][`crate::adf::Adf::print_interpretation`].
#[derive(Debug)]
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
                write!(
                    f,
                    "{}) ",
                    self.ordering
                        .name(Var(pos))
                        .expect("Variable originates from same parser object as the ordering")
                )
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

/// Provides an Iterator, which contains all three valued Interpretations, with respect to the given
/// 3-valued interpretation.

#[derive(Debug)]
pub struct ThreeValuedInterpretationsIterator {
    original: Vec<Term>,
    indexes: Vec<usize>,
    current: Option<Vec<usize>>,
    started: bool,
}

impl ThreeValuedInterpretationsIterator {
    /// Creates a new iterable structure, which represents all three-valued interpretations wrt. the given 3-valued interpretation
    pub fn new(term: &[Term]) -> Self {
        let indexes = term
            .iter()
            .enumerate()
            .filter_map(|(idx, &v)| (!v.is_truth_value()).then(|| idx))
            .rev()
            .collect::<Vec<_>>();
        let current = vec![2; indexes.len()];

        Self {
            indexes,
            started: false,
            current: Some(current),
            original: term.into(),
        }
    }

    fn decrement(&mut self) {
        if let Some(ref mut current) = self.current {
            if !ThreeValuedInterpretationsIterator::decrement_vec(current) {
                self.current = None;
            }
        }
    }

    fn decrement_vec(vector: &mut Vec<usize>) -> bool {
        let mut cur_pos = None;
        for (idx, value) in vector.iter_mut().enumerate() {
            if *value > 0 {
                *value -= 1;
                cur_pos = Some(idx);
                break;
            }
        }
        if let Some(cur) = cur_pos {
            for value in vector[0..cur].iter_mut() {
                *value = 2;
            }
            true
        } else {
            false
        }
    }
}

impl Iterator for ThreeValuedInterpretationsIterator {
    type Item = Vec<Term>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.started {
            if self.current.is_some() {
                self.decrement();
            }
        } else {
            self.started = true;
        }
        if let Some(current) = &self.current {
            let mut result = self.original.clone();
            for (idx, val) in current.iter().enumerate() {
                result[self.indexes[idx]] = match val {
                    0 => Term::BOT,
                    1 => Term::TOP,
                    _ => self.original[self.indexes[idx]],
                }
            }
            return Some(result);
        }
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;

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

    #[test]
    fn three_valued_interpretations() {
        let testinterpretation = vec![Term::TOP, Term(22), Term::BOT, Term(12), Term::TOP];
        let mut iter = ThreeValuedInterpretationsIterator::new(&testinterpretation);
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term(22), Term::BOT, Term(12), Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term(22), Term::BOT, Term::TOP, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term(22), Term::BOT, Term::BOT, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::TOP, Term::BOT, Term(12), Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::TOP, Term::BOT, Term::TOP, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::TOP, Term::BOT, Term::BOT, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::BOT, Term::BOT, Term(12), Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::BOT, Term::BOT, Term::TOP, Term::TOP])
        );
        assert_eq!(
            iter.next(),
            Some(vec![Term::TOP, Term::BOT, Term::BOT, Term::BOT, Term::TOP])
        );
        assert_eq!(iter.next(), None);

        let testinterpretation = vec![Term(1), Term(3), Term(3), Term(7)];
        let iter: Vec<Vec<Term>> =
            ThreeValuedInterpretationsIterator::new(&testinterpretation).collect();
        assert_eq!(
            iter,
            [
                [Term(1), Term(3), Term(3), Term(7)],
                [Term(1), Term(3), Term(3), Term(1)],
                [Term(1), Term(3), Term(3), Term(0)],
                [Term(1), Term(3), Term(1), Term(7)],
                [Term(1), Term(3), Term(1), Term(1)],
                [Term(1), Term(3), Term(1), Term(0)],
                [Term(1), Term(3), Term(0), Term(7)],
                [Term(1), Term(3), Term(0), Term(1)],
                [Term(1), Term(3), Term(0), Term(0)],
                [Term(1), Term(1), Term(3), Term(7)],
                [Term(1), Term(1), Term(3), Term(1)],
                [Term(1), Term(1), Term(3), Term(0)],
                [Term(1), Term(1), Term(1), Term(7)],
                [Term(1), Term(1), Term(1), Term(1)],
                [Term(1), Term(1), Term(1), Term(0)],
                [Term(1), Term(1), Term(0), Term(7)],
                [Term(1), Term(1), Term(0), Term(1)],
                [Term(1), Term(1), Term(0), Term(0)],
                [Term(1), Term(0), Term(3), Term(7)],
                [Term(1), Term(0), Term(3), Term(1)],
                [Term(1), Term(0), Term(3), Term(0)],
                [Term(1), Term(0), Term(1), Term(7)],
                [Term(1), Term(0), Term(1), Term(1)],
                [Term(1), Term(0), Term(1), Term(0)],
                [Term(1), Term(0), Term(0), Term(7)],
                [Term(1), Term(0), Term(0), Term(1)],
                [Term(1), Term(0), Term(0), Term(0)]
            ]
        );
    }

    #[test]
    fn tvi_decrement() {
        let testinterpretation = vec![Term::TOP, Term(22), Term::BOT, Term(12), Term::TOP];
        let mut iter = ThreeValuedInterpretationsIterator::new(&testinterpretation);
        assert_eq!(iter.current, Some(vec![2, 2]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![1, 2]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![0, 2]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![2, 1]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![1, 1]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![0, 1]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![2, 0]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![1, 0]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![0, 0]));
        iter.decrement();
        assert_eq!(iter.current, None);

        let testinterpretation = vec![Term::TOP, Term(22), Term::BOT, Term::TOP, Term::TOP];
        let mut iter = ThreeValuedInterpretationsIterator::new(&testinterpretation);

        assert_eq!(iter.current, Some(vec![2]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![1]));
        iter.decrement();
        assert_eq!(iter.current, Some(vec![0]));
        iter.decrement();
        assert_eq!(iter.current, None);
    }
}
