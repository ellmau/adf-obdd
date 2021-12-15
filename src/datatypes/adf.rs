use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::obdd::Bdd;
use crate::parser::{AdfParser, Formula};

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
}

pub struct Adf {
    ordering: VarContainer,
    bdd: Bdd,
    ac: Vec<Term>,
}

impl Adf {
    pub fn from_parser(parser: &AdfParser) -> Self {
        let mut result = Self {
            ordering: VarContainer::from_parser(
                parser.namelist_rc_refcell(),
                parser.dict_rc_refcell(),
            ),
            bdd: Bdd::new(),
            ac: Vec::new(),
        };
        (0..parser.namelist_rc_refcell().borrow().len())
            .into_iter()
            .for_each(|value| {
                result.bdd.variable(Var(value));
            });

        parser.formula_order().iter().for_each(|pos| {
            let result_term = result.term(&parser.ac_at(*pos).unwrap());
            result.ac.push(result_term);
        });
        result
    }

    fn term(&mut self, formula: &Formula) -> Term {
        match formula {
            Formula::Bot => Bdd::constant(false),
            Formula::Top => Bdd::constant(true),
            Formula::Atom(val) => {
                let t1 = self.ordering.variable(val).unwrap();
                self.bdd.variable(t1)
            }
            Formula::Not(val) => {
                let t1 = self.term(val);
                self.bdd.not(t1)
            }
            Formula::And(val1, val2) => {
                let t1 = self.term(val1);
                let t2 = self.term(val2);
                self.bdd.and(t1, t2)
            }
            Formula::Or(val1, val2) => {
                let t1 = self.term(val1);
                let t2 = self.term(val2);
                self.bdd.or(t1, t2)
            }
            Formula::Iff(val1, val2) => {
                let t1 = self.term(val1);
                let t2 = self.term(val2);
                self.bdd.iff(t1, t2)
            }
            Formula::Xor(val1, val2) => {
                let t1 = self.term(val1);
                let t2 = self.term(val2);
                self.bdd.xor(t1, t2)
            }
            Formula::Imp(val1, val2) => {
                let t1 = self.term(val1);
                let t2 = self.term(val2);
                self.bdd.imp(t1, t2)
            }
        }
    }

    pub fn complete(&mut self) -> Vec<Term> {
        let mut t_vals: usize =
            self.ac.iter().fold(
                0,
                |acc, elem| {
                    if elem.is_truth_value() {
                        acc + 1
                    } else {
                        acc
                    }
                },
            );
        let mut new_interpretation = self.ac.clone();
        loop {
            let old_t_vals = t_vals;
            for ac in new_interpretation
                .iter_mut()
                .filter(|term| !term.is_truth_value())
            {
                *ac = self.ac.iter().enumerate().fold(*ac, |acc, (var, term)| {
                    if term.is_truth_value() {
                        self.bdd.restrict(acc, Var(var), term.is_true())
                    } else {
                        acc
                    }
                });
                if ac.is_truth_value() {
                    t_vals += 1;
                }
            }
            if t_vals == old_t_vals {
                break;
            }
        }
        new_interpretation
    }

    pub fn print_interpretation<'a, 'b>(
        &'a self,
        interpretation: &'b Vec<Term>,
    ) -> PrintableInterpretation<'b>
    where
        'a: 'b,
    {
        PrintableInterpretation {
            interpretation,
            ordering: &self.ordering,
        }
    }
}

pub struct PrintableInterpretation<'a> {
    interpretation: &'a Vec<Term>,
    ordering: &'a VarContainer,
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
    fn from_parser() {
        let parser = AdfParser::default();
        let input = "s(a).s(c).ac(a,b).ac(b,neg(a)).s(b).ac(c,and(c(v),or(c(f),a))).s(e).s(d).ac(d,iff(imp(a,b),c)).ac(e,xor(d,e)).";

        parser.parse()(input).unwrap();

        let adf = Adf::from_parser(&parser);
        assert_eq!(adf.ordering.names.as_ref().borrow()[0], "a");
        assert_eq!(adf.ordering.names.as_ref().borrow()[1], "c");
        assert_eq!(adf.ordering.names.as_ref().borrow()[2], "b");
        assert_eq!(adf.ordering.names.as_ref().borrow()[3], "e");
        assert_eq!(adf.ordering.names.as_ref().borrow()[4], "d");

        assert_eq!(adf.ac, vec![Term(4), Term(2), Term(7), Term(10), Term(15)]);
    }
    #[test]
    fn init_varcontainer() {
        let vc = VarContainer::default();
        assert_eq!(vc.variable("foo"), None);
    }

    #[test]
    fn complete() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).s(e).ac(e,and(b,or(neg(b),c(f)))).s(f).ac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);
        let result = adf.complete();

        assert_eq!(
            result,
            vec![Term(1), Term(3), Term(3), Term(9), Term(0), Term(1)]
        );
        assert_eq!(
            format!("{}", adf.print_interpretation(&result)),
            "T(a) u(b) u(c) u(d) F(e) T(f) \n"
        );
    }
}
