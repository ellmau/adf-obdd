//! This module describes the abstract dialectical framework
//!
//!  - computing interpretations
//!  - computing fixpoints

use serde::{Deserialize, Serialize};

use crate::{
    datatypes::{
        adf::{
            PrintDictionary, PrintableInterpretation, ThreeValuedInterpretationsIterator,
            TwoValuedInterpretationsIterator, VarContainer,
        },
        FacetCounts, ModelCounts, Term, Var,
    },
    obdd::Bdd,
    parser::{AdfParser, Formula},
};

#[derive(Serialize, Deserialize, Debug)]
/// Representation of an ADF, with an ordering and dictionary of statement <-> number relations, a binary decision diagram, and a list of acceptance functions in Term representation
pub struct Adf {
    ordering: VarContainer,
    bdd: Bdd,
    ac: Vec<Term>,
}

impl Default for Adf {
    fn default() -> Self {
        Self {
            ordering: VarContainer::default(),
            bdd: Bdd::new(),
            ac: Vec::new(),
        }
    }
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

    pub(crate) fn from_biodivine_vector(
        ordering: &VarContainer,
        bio_ac: &[biodivine_lib_bdd::Bdd],
    ) -> Self {
        let mut result = Self {
            ordering: VarContainer::copy(ordering),
            bdd: Bdd::new(),
            ac: vec![Term(0); bio_ac.len()],
        };
        result
            .ac
            .iter_mut()
            .zip(bio_ac.iter())
            .for_each(|(new_ac, bdd_ac)| {
                if bdd_ac.is_true() {
                    *new_ac = Bdd::constant(true);
                } else if bdd_ac.is_false() {
                    *new_ac = Bdd::constant(false);
                } else {
                    // compound formula
                    let mut term_vec: Vec<Term> = Vec::new();
                    for (idx, tuple) in bdd_ac
                        .to_string()
                        .split('|')
                        .filter(|tuple| !tuple.is_empty())
                        .enumerate()
                    {
                        let node_elements = tuple.split(',').collect::<Vec<&str>>();
                        if idx == 0 {
                            term_vec.push(Bdd::constant(false));
                        } else if idx == 1 {
                            term_vec.push(Bdd::constant(true));
                        } else {
                            let new_term = result.bdd.node(
                                Var(node_elements[0]
                                    .parse::<usize>()
                                    .expect("Var should be number")),
                                term_vec[node_elements[1]
                                    .parse::<usize>()
                                    .expect("Termpos should be a valid number")],
                                term_vec[node_elements[2]
                                    .parse::<usize>()
                                    .expect("Termpos should be a valid number")],
                            );
                            term_vec.push(new_term);
                        }
                        *new_ac = *term_vec
                            .last()
                            .expect("There should be one element in the vector");
                    }
                }
            });
        result
    }

    /// Instantiates a new ADF, based on a biodivine adf
    pub fn from_biodivine(bio_adf: &super::adfbiodivine::Adf) -> Self {
        Self::from_biodivine_vector(bio_adf.var_container(), bio_adf.ac())
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
        log::info!("[Start] grounded");
        let ac = &self.ac.clone();
        let result = self.grounded_internal(ac);
        log::info!("[Done] grounded");
        result
    }

    fn grounded_internal(&mut self, interpretation: &[Term]) -> Vec<Term> {
        let mut t_vals: usize = interpretation
            .iter()
            .filter(|elem| elem.is_truth_value())
            .count();
        let mut new_interpretation: Vec<Term> = interpretation.into();
        loop {
            let curr_interpretation = new_interpretation.clone();
            let old_t_vals = t_vals;
            for ac in new_interpretation
                .iter_mut()
                .filter(|term| !term.is_truth_value())
            {
                *ac = curr_interpretation
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
            log::debug!(
                "old-int: {:?}, {} constants",
                curr_interpretation,
                old_t_vals
            );
            log::debug!("new-int: {:?}, {} constants", new_interpretation, t_vals);
            if t_vals == old_t_vals {
                break;
            }
        }
        new_interpretation
    }

    /// Computes the stable models
    /// Returns an Iterator which contains all stable models
    pub fn stable<'a, 'c>(&'a mut self) -> impl Iterator<Item = Vec<Term>> + 'c
    where
        'a: 'c,
    {
        let grounded = self.grounded();
        TwoValuedInterpretationsIterator::new(&grounded)
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

    /// Computes the stable models
    /// Returns a vector with all stable models, using a single-formula representation in biodivine to enumerate the possible models
    /// Note that the biodivine adf needs to be the one which instantiated the adf (if applicable)
    pub fn stable_bdd_representation(
        &mut self,
        biodivine: &crate::adfbiodivine::Adf,
    ) -> Vec<Vec<Term>> {
        biodivine
            .stable_model_candidates()
            .into_iter()
            .filter(|terms| {
                let mut interpr = self.ac.clone();
                for ac in interpr.iter_mut() {
                    *ac = terms.iter().enumerate().fold(*ac, |acc, (var, term)| {
                        if term.is_truth_value() && !term.is_true() {
                            self.bdd.restrict(acc, Var(var), false)
                        } else {
                            acc
                        }
                    });
                }
                let grounded_check = self.grounded_internal(&interpr);
                terms
                    .iter()
                    .zip(grounded_check.iter())
                    .all(|(left, right)| left.compare_inf(right))
            })
            .collect::<Vec<Vec<Term>>>()
    }

    /// Computes the stable models
    /// Returns an Iterator which contains all stable models
    pub fn stable_with_prefilter<'a, 'c>(&'a mut self) -> impl Iterator<Item = Vec<Term>> + 'c
    where
        'a: 'c,
    {
        let grounded = self.grounded();
        TwoValuedInterpretationsIterator::new(&grounded)
            .map(|interpretation| {
                if interpretation.iter().enumerate().all(|(ac_idx, it)| {
                    it.compare_inf(&interpretation.iter().enumerate().fold(
                        self.ac[ac_idx],
                        |acc, (var, term)| {
                            if term.is_truth_value() {
                                self.bdd.restrict(acc, Var(var), term.is_true())
                            } else {
                                acc
                            }
                        },
                    ))
                }) {
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
                } else {
                    (vec![Term::BOT], vec![Term::TOP])
                }
            })
            .filter(|(int, grd)| {
                int.iter()
                    .zip(grd.iter())
                    .all(|(it, gr)| it.compare_inf(gr))
            })
            .map(|(int, _grd)| int)
    }

    /// Computes the complete models
    /// Returns an Iterator which contains all complete models
    pub fn complete<'a, 'c>(&'a mut self) -> impl Iterator<Item = Vec<Term>> + 'c
    where
        'a: 'c,
    {
        let grounded = self.grounded();
        let ac = self.ac.clone();
        ThreeValuedInterpretationsIterator::new(&grounded).filter(move |interpretation| {
            interpretation.iter().enumerate().all(|(ac_idx, it)| {
                log::trace!("idx [{}], term: {}", ac_idx, it);
                it.compare_inf(&interpretation.iter().enumerate().fold(
                    ac[ac_idx],
                    |acc, (var, term)| {
                        if term.is_truth_value() {
                            self.bdd.restrict(acc, Var(var), term.is_true())
                        } else {
                            acc
                        }
                    },
                ))
            })
        })
    }

    /// Returns a [Vector][std::vec::Vec] of [ModelCounts][crate::datatypes::ModelCounts] for each acceptance condition.
    ///
    /// `memoization` controls whether memoization is utilised or not.
    pub fn formulacounts(&self, memoization: bool) -> Vec<ModelCounts> {
        self.ac
            .iter()
            .map(|ac| self.bdd.models(*ac, memoization))
            .collect()
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

    /// creates a [PrintDictionary] for output purposes
    pub fn print_dictionary(&self) -> PrintDictionary {
        PrintDictionary::new(&self.ordering)
    }

    /// Fixes the bdd after an import with serde
    pub fn fix_import(&mut self) {
        self.bdd.fix_import();
    }

    /// Counts facets of respective [Terms][crate::datatypes::Term]
    /// and returns [Vector][std::vec::Vec] containing respective
    /// facet counts.
    pub fn facet_count(&self, interpretation: &[Term]) -> Vec<(ModelCounts, FacetCounts)> {
        interpretation
            .iter()
            .map(|t| {
                let mcs = self.bdd.models(*t, true);

                let n_vdps = { |t| self.bdd.var_dependencies(t).len() };

                let fc = match mcs.1 > 2 {
                    true => 2 * n_vdps(*t),
                    _ => 0,
                };
                let cfc = match mcs.0 > 2 {
                    true => 2 * n_vdps(*t),
                    _ => 0,
                };
                (mcs, (cfc, fc))
            })
            .collect::<Vec<_>>()
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
    fn serialize() {
        let parser = AdfParser::default();
        let input = "s(a).s(c).ac(a,b).ac(b,neg(a)).s(b).ac(c,and(c(v),or(c(f),a))).s(e).s(d).ac(d,iff(imp(a,b),c)).ac(e,xor(d,e)).";

        parser.parse()(input).unwrap();
        let mut adf = Adf::from_parser(&parser);

        let grounded = adf.grounded();

        let serialized = serde_json::to_string(&adf).unwrap();
        log::debug!("Serialized to {}", serialized);
        let result = r#"{"ordering":{"names":["a","c","b","e","d"],"mapping":{"b":2,"a":0,"c":1,"e":3,"d":4}},"bdd":{"nodes":[{"var":18446744073709551614,"lo":0,"hi":0},{"var":18446744073709551615,"lo":1,"hi":1},{"var":0,"lo":0,"hi":1},{"var":1,"lo":0,"hi":1},{"var":2,"lo":0,"hi":1},{"var":3,"lo":0,"hi":1},{"var":4,"lo":0,"hi":1},{"var":0,"lo":1,"hi":0},{"var":0,"lo":1,"hi":4},{"var":1,"lo":1,"hi":0},{"var":2,"lo":1,"hi":0},{"var":1,"lo":10,"hi":4},{"var":0,"lo":3,"hi":11},{"var":3,"lo":1,"hi":0},{"var":4,"lo":1,"hi":0},{"var":3,"lo":6,"hi":14}],"cache":[[{"var":1,"lo":0,"hi":1},3],[{"var":3,"lo":6,"hi":14},15],[{"var":2,"lo":0,"hi":1},4],[{"var":0,"lo":1,"hi":0},7],[{"var":0,"lo":3,"hi":11},12],[{"var":3,"lo":1,"hi":0},13],[{"var":4,"lo":1,"hi":0},14],[{"var":0,"lo":0,"hi":1},2],[{"var":3,"lo":0,"hi":1},5],[{"var":0,"lo":1,"hi":4},8],[{"var":4,"lo":0,"hi":1},6],[{"var":1,"lo":1,"hi":0},9],[{"var":2,"lo":1,"hi":0},10],[{"var":1,"lo":10,"hi":4},11]],"count_cache":{}},"ac":[4,2,7,15,12]}"#;
        let mut deserialized: Adf = serde_json::from_str(result).unwrap();
        assert_eq!(adf.ac, deserialized.ac);
        let grounded_import = deserialized.grounded();

        assert_eq!(grounded, grounded_import);
        assert_eq!(
            format!("{}", adf.print_interpretation(&grounded)),
            format!("{}", deserialized.print_interpretation(&grounded_import))
        );
    }

    #[test]
    fn grounded() {
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

        let parser = AdfParser::default();
        parser.parse()(
            "s(a).s(b).s(c).s(d).s(e).ac(a,c(v)).ac(b,a).ac(c,b).ac(d,neg(c)).ac(e,and(a,d)).",
        )
        .unwrap();
        let mut adf = Adf::from_parser(&parser);
        let result = adf.grounded();
        assert_eq!(result, vec![Term(1), Term(1), Term(1), Term(0), Term(0)]);
    }

    #[test]
    fn stable() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);

        let mut stable = adf.stable();
        assert_eq!(
            stable.next(),
            Some(vec![
                Term::TOP,
                Term::BOT,
                Term::BOT,
                Term::TOP,
                Term::BOT,
                Term::TOP
            ])
        );
        assert_eq!(stable.next(), None);

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,neg(b)).ac(b,neg(a)).").unwrap();
        let mut adf = Adf::from_parser(&parser);
        let mut stable = adf.stable();

        assert_eq!(stable.next(), Some(vec![Term::BOT, Term::TOP]));
        assert_eq!(stable.next(), Some(vec![Term::TOP, Term::BOT]));
        assert_eq!(stable.next(), None);

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,b).ac(b,a).").unwrap();
        let mut adf = Adf::from_parser(&parser);

        assert_eq!(
            adf.stable().collect::<Vec<_>>(),
            vec![vec![Term::BOT, Term::BOT]]
        );

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,neg(a)).ac(b,a).").unwrap();
        let mut adf = Adf::from_parser(&parser);
        assert_eq!(adf.stable().next(), None);
    }

    #[test]
    fn complete() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);

        assert_eq!(
            adf.complete().next(),
            Some(vec![Term(1), Term(3), Term(3), Term(9), Term(0), Term(1)])
        );

        assert_eq!(
            adf.complete().collect::<Vec<_>>(),
            [
                [Term(1), Term(3), Term(3), Term(9), Term(0), Term(1)],
                [Term(1), Term(1), Term(1), Term(0), Term(0), Term(1)],
                [Term(1), Term(0), Term(0), Term(1), Term(0), Term(1)]
            ]
        );
    }

    #[test]
    fn complete2() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);
        assert_eq!(
            adf.complete().collect::<Vec<_>>(),
            [
                [Term(1), Term(3), Term(3), Term(7)],
                [Term(1), Term(1), Term(1), Term(0)],
                [Term(1), Term(0), Term(0), Term(1)]
            ]
        );
        let printer = adf.print_dictionary();
        for model in adf.complete() {
            println!("{}", printer.print_interpretation(&model));
        }
    }

    #[test]
    fn formulacounts() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).")
            .unwrap();
        let adf = Adf::from_parser(&parser);

        assert_eq!(adf.formulacounts(false), adf.formulacounts(true));
    }

    #[test]
    fn adf_default() {
        let _adf = Adf::default();
    }

    #[test]
    fn facet_counts() {
        let parser = AdfParser::default();
        parser.parse()(
            "s(a). s(b). s(c). s(d). ac(a,c(v)). ac(b,b). ac(c,and(a,b)). ac(d,neg(b)).",
        )
        .unwrap();
        let mut adf = Adf::from_parser(&parser);

        let mut v = adf.ac.clone();
        let mut fcs = adf.facet_count(&v);
        assert_eq!(
            fcs.iter().map(|t| t.1).collect::<Vec<_>>(),
            vec![(0, 0), (0, 0), (4, 0), (0, 0)]
        );

        v[0] = Term::TOP;
        // make navigation step for each bdd in adf-bdd-represenation
        v = v
            .iter()
            .map(|t| {
                v.iter()
                    .enumerate()
                    .fold(*t, |acc, (var, term)| match term.is_truth_value() {
                        true => adf.bdd.restrict(acc, Var(var), term.is_true()),
                        _ => acc,
                    })
            })
            .collect::<Vec<_>>();
        fcs = adf.facet_count(&v);

        assert_eq!(
            fcs.iter().map(|t| t.1).collect::<Vec<_>>(),
            vec![(0, 0), (0, 0), (0, 0), (0, 0)]
        );
    }
}
