use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::Var;

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
        VarContainer(names, mapping)
    }

    pub fn name(&self, variable: Var) -> Option<&str> {
        self.names.borrow().get(variable)
    }

    pub fn variable(&self, name: &str) -> Option<Var> {
        self.mapping.borrow().get(name).and_then(|val| Var(*val))
    }
}
