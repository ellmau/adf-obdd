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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn init_varcontainer() {
        let vc = VarContainer::default();
        assert_eq!(vc.variable("foo"), None);
    }
}
