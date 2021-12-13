use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use super::super::parser::AdfParser;
use crate::obdd::Bdd;

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
}

pub struct Adf {
    ordering: VarContainer,
    bdd: Bdd,
    ac: Vec<Term>,
}

impl Adf {
    fn from_parser(parser: &AdfParser) -> Self {
        let mut result = Self {
            ordering: VarContainer::from_parser(
                parser.namelist_rc_refcell(),
                parser.dict_rc_refcell(),
            ),
            bdd: Bdd::new(),
            ac: Vec::with_capacity(parser.formula_count()),
        };
        let ac_order: Vec<usize> = Vec::new();

        (0..parser.namelist_rc_refcell().borrow().len())
            .into_iter()
            .for_each(|value| {
                result.bdd.variable(Var(value));
            });
        result
    }
}
