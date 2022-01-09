//! This module describes the abstract dialectical framework
//!
//! It handles
//!  - parsing of statements and acceptance functions
//!  - computing interpretations
//!  - computing fixpoints
//!  - computing the least fixpoint by using a shortcut

use crate::{
    datatypes::{
        adf::{PrintableInterpretation, TwoValuedInterpretationsIterator, VarContainer},
        Term, Var,
    },
    obdd::Bdd,
    parser::{AdfParser, Formula},
};
/// Representation of an ADF, with an ordering and dictionary of statement <-> number relations, a binary decision diagram, and a list of acceptance functions in Term representation
pub struct Adf {
    ordering: VarContainer,
    bdd: Bdd,
    ac: Vec<Term>,
}

impl Adf {
    /// Instantiates a new ADF, based on the parser-data
    pub fn from_parser(parser: &AdfParser) -> Self {
        log::info!("[Start] instantiating BDD");
        let mut result = Self {
            ordering: VarContainer::from_parser(
                parser.namelist_rc_refcell(),
                parser.dict_rc_refcell(),
            ),
            bdd: Bdd::new(),
            ac: vec![Term(0); parser.namelist_rc_refcell().as_ref().borrow().len()],
        };
        (0..parser.namelist_rc_refcell().borrow().len())
            .into_iter()
            .for_each(|value| {
                log::trace!("adding variable {}", Var(value));
                result.bdd.variable(Var(value));
            });
        log::debug!("[Start] adding acs");
        parser
            .formula_order()
            .iter()
            .enumerate()
            .for_each(|(insert_order, new_order)| {
                log::trace!(
                    "Pos {}/{} formula {}, {:?}",
                    insert_order + 1,
                    parser.formula_count(),
                    new_order,
                    parser.ac_at(insert_order)
                );
                let result_term = result.term(&parser.ac_at(insert_order).unwrap());
                result.ac[*new_order] = result_term;
            });
        log::info!("[Success] instantiated");
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

    /// Computes the grounded extension and returns it as a list
    pub fn grounded(&mut self) -> Vec<Term> {
        let ac = &self.ac.clone();
        self.grounded_internal(ac)
    }

    fn grounded_internal(&mut self, interpretation: &[Term]) -> Vec<Term> {
        log::info!("[Start] grounded");
        let mut t_vals: usize = interpretation
            .iter()
            .filter(|elem| elem.is_truth_value())
            .count();
        let mut new_interpretation: Vec<Term> = interpretation.into();
        loop {
            let old_t_vals = t_vals;
            for ac in new_interpretation
                .iter_mut()
                .filter(|term| !term.is_truth_value())
            {
                *ac = interpretation
                    .iter()
                    .enumerate()
                    .fold(*ac, |acc, (var, term)| {
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
        log::info!("[Done] grounded");
        new_interpretation
    }

    /// Computes the first `max_values` stable models
    /// if max_values is 0, then all will be computed
    pub fn stable(&mut self, max_values: usize) -> Vec<Vec<Term>> {
        let grounded = self.grounded();
        if max_values == 0 {
            self.stable_iter(&grounded).collect()
        } else {
            self.stable_iter(&grounded)
                .enumerate()
                .take_while(|(idx, _elem)| *idx < max_values)
                .map(|(_, elem)| elem)
                .collect()
        }
    }

    /// Computes the stable models
    /// Returns an Iterator which contains all stable models
    fn stable_iter<'a, 'b, 'c>(
        &'a mut self,
        grounded: &'b [Term],
    ) -> impl Iterator<Item = Vec<Term>> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        TwoValuedInterpretationsIterator::new(grounded)
            .map(|interpretation| {
                let mut interpr = self.ac.clone();
                for ac in interpr.iter_mut() {
                    *ac = interpretation
                        .iter()
                        .enumerate()
                        .fold(*ac, |acc, (var, term)| {
                            if term.is_truth_value() && !term.is_true() {
                                self.bdd.restrict(acc, Var(var), false)
                            } else {
                                acc
                            }
                        });
                }
                let grounded_check = self.grounded_internal(&interpr);
                log::debug!(
                    "grounded candidate\n{:?}\n{:?}",
                    interpretation,
                    grounded_check
                );
                (interpretation, grounded_check)
            })
            .filter(|(int, grd)| {
                int.iter()
                    .zip(grd.iter())
                    .all(|(it, gr)| it.compare_inf(gr))
            })
            .map(|(int, _grd)| int)
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

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;
    #[test]
    fn from_parser() {
        let parser = AdfParser::default();
        let input = "s(a).s(c).ac(a,b).ac(b,neg(a)).s(b).ac(c,and(c(v),or(c(f),a))).s(e).s(d).ac(d,iff(imp(a,b),c)).ac(e,xor(d,e)).";

        parser.parse()(input).unwrap();

        let adf = Adf::from_parser(&parser);
        assert_eq!(adf.ordering.names().as_ref().borrow()[0], "a");
        assert_eq!(adf.ordering.names().as_ref().borrow()[1], "c");
        assert_eq!(adf.ordering.names().as_ref().borrow()[2], "b");
        assert_eq!(adf.ordering.names().as_ref().borrow()[3], "e");
        assert_eq!(adf.ordering.names().as_ref().borrow()[4], "d");

        assert_eq!(adf.ac, vec![Term(4), Term(2), Term(7), Term(15), Term(12)]);

        let parser = AdfParser::default();
        let input = "s(a).s(c).ac(a,b).ac(b,neg(a)).s(b).ac(c,and(c(v),or(c(f),a))).s(e).s(d).ac(d,iff(imp(a,b),c)).ac(e,xor(d,e)).";

        parser.parse()(input).unwrap();
        parser.varsort_alphanum();

        let adf = Adf::from_parser(&parser);
        assert_eq!(adf.ordering.names().as_ref().borrow()[0], "a");
        assert_eq!(adf.ordering.names().as_ref().borrow()[1], "b");
        assert_eq!(adf.ordering.names().as_ref().borrow()[2], "c");
        assert_eq!(adf.ordering.names().as_ref().borrow()[3], "d");
        assert_eq!(adf.ordering.names().as_ref().borrow()[4], "e");

        assert_eq!(adf.ac, vec![Term(3), Term(7), Term(2), Term(11), Term(13)]);
    }

    #[test]
    fn complete() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);
        let result = adf.grounded();

        assert_eq!(
            result,
            vec![Term(1), Term(3), Term(3), Term(9), Term(0), Term(1)]
        );
        assert_eq!(
            format!("{}", adf.print_interpretation(&result)),
            "T(a) u(b) u(c) u(d) F(e) T(f) \n"
        );
    }

    #[test]
    fn stable() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);

        assert_eq!(
            adf.stable(0),
            vec![vec![
                Term::TOP,
                Term::BOT,
                Term::BOT,
                Term::TOP,
                Term::BOT,
                Term::TOP
            ]]
        );
        assert_eq!(
            adf.stable(10),
            vec![vec![
                Term::TOP,
                Term::BOT,
                Term::BOT,
                Term::TOP,
                Term::BOT,
                Term::TOP
            ]]
        );

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,neg(b)).ac(b,neg(a)).").unwrap();
        let mut adf = Adf::from_parser(&parser);

        assert_eq!(adf.stable(1), vec![vec![Term::BOT, Term::TOP]]);
        assert_eq!(adf.stable(2), adf.stable(0));
        assert_eq!(
            adf.stable(0),
            vec![vec![Term::BOT, Term::TOP], vec![Term::TOP, Term::BOT]]
        );

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,b).ac(b,a).").unwrap();
        let mut adf = Adf::from_parser(&parser);

        assert_eq!(adf.stable(0), vec![vec![Term::BOT, Term::BOT]]);

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,neg(a)).ac(b,a).").unwrap();
        let mut adf = Adf::from_parser(&parser);

        let empty: Vec<Vec<Term>> = Vec::new();
        assert_eq!(adf.stable(0), empty);
        assert_eq!(adf.stable(99999), empty);
    }
}
