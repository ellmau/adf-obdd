/*!
This module describes the abstract dialectical framework.

 - computing interpretations and models
 - computing fixpoints
*/

pub mod heuristics;
use crate::{
    datatypes::{
        adf::{
            PrintDictionary, PrintableInterpretation, ThreeValuedInterpretationsIterator,
            TwoValuedInterpretationsIterator, VarContainer,
        },
        FacetCounts, ModelCounts, Term, Var,
    },
    nogoods::{NoGood, NoGoodStore},
    obdd::Bdd,
    parser::{AdfParser, Formula},
};
use serde::{Deserialize, Serialize};

use self::heuristics::Heuristic;

#[derive(Serialize, Deserialize, Debug)]
/// Representation of an ADF, with an ordering and dictionary which relates statements to numbers, a binary decision diagram, and a list of acceptance conditions in [`Term`][crate::datatypes::Term] representation.
///
/// Please note that due to the nature of the underlying reduced and ordered Bdd the concept of a [`Term`][crate::datatypes::Term] represents one (sub) formula as well as truth-values.
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
    /// Instantiates a new ADF, based on the [parser-data][crate::parser::AdfParser].
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
                let result_term = result.term(&parser.ac_at(insert_order).expect(
                    "Index should exist, because the data originates from the same parser object",
                ));
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
        log::trace!("ordering: {:?}", result.ordering);
        log::trace!("adf {:?} instantiated with bdd {}", result.ac, result.bdd);
        result
    }

    /// Instantiates a new ADF, based on a [biodivine adf][crate::adfbiodivine::Adf].
    pub fn from_biodivine(bio_adf: &super::adfbiodivine::Adf) -> Self {
        Self::from_biodivine_vector(bio_adf.var_container(), bio_adf.ac())
    }

    fn term(&mut self, formula: &Formula) -> Term {
        match formula {
            Formula::Bot => Bdd::constant(false),
            Formula::Top => Bdd::constant(true),
            Formula::Atom(val) => {
                let t1 = self.ordering.variable(val).expect("Variable should exist, because the ordering has been filled by the same parser as the input formula comes from");
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

    /// Computes the grounded extension and returns it as a list.
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

    /// Computes the stable models.
    /// Returns an Iterator which contains all stable models.
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

    /// Computes the stable models.
    /// Returns a vector with all stable models, using a single-formula representation in biodivine to enumerate the possible models.
    /// Note that the biodivine adf needs to be the one which instantiated the adf (if applicable).
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

    /// Computes the stable models.
    /// Returns an Iterator which contains all stable models.
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

    /// Computes the stable models.
    /// Returns an iterator which contains all stable models.
    /// This variant uses the heuristic, which uses maximal [var impact][crate::obdd::Bdd::passive_var_impact], minimal [self-cycle impact][crate::obdd::Bdd::active_var_impact] and the minimal amount of [paths][crate::obdd::Bdd::paths].
    pub fn stable_count_optimisation_heu_a<'a, 'c>(
        &'a mut self,
    ) -> impl Iterator<Item = Vec<Term>> + 'c
    where
        'a: 'c,
    {
        log::debug!("[Start] stable count optimisation");
        let grounded = self.grounded();
        self.two_val_model_counts(&grounded, Self::heu_max_imp_min_nacyc_impact_min_paths)
            .into_iter()
            .filter(|int| self.stability_check(int))
    }

    /// Computes the stable models.
    /// Returns an iterator which contains all stable models.
    /// This variant uses the heuristic, which uses minimal number of [paths][crate::obdd::Bdd::paths] and maximal [variable-impact][crate::obdd::Bdd::passive_var_impact].
    pub fn stable_count_optimisation_heu_b<'a, 'c>(
        &'a mut self,
    ) -> impl Iterator<Item = Vec<Term>> + 'c
    where
        'a: 'c,
    {
        log::debug!("[Start] stable count optimisation");
        let grounded = self.grounded();
        self.two_val_model_counts(&grounded, Self::heu_min_paths_max_imp)
            .into_iter()
            .filter(|int| self.stability_check(int))
    }

    fn stability_check(&mut self, interpretation: &[Term]) -> bool {
        let mut new_int = self.ac.clone();
        for ac in new_int.iter_mut() {
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

        let grd = self.grounded_internal(&new_int);
        for (idx, grd) in grd.iter().enumerate() {
            if !grd.compare_inf(&interpretation[idx]) {
                return false;
            }
        }
        true
    }

    fn is_two_valued(&self, interpretation: &[Term]) -> bool {
        interpretation.iter().all(|t| t.is_truth_value())
    }

    fn two_val_model_counts<H>(&mut self, interpr: &[Term], heuristic: H) -> Vec<Vec<Term>>
    where
        H: Fn(&Self, (Var, Term), (Var, Term), &[Term]) -> std::cmp::Ordering + Copy,
    {
        self.two_val_model_counts_logic(interpr, &vec![Term::UND; interpr.len()], 0, heuristic)
    }

    fn heu_max_imp_min_nacyc_impact_min_paths(
        &self,
        lhs: (Var, Term),
        rhs: (Var, Term),
        interpr: &[Term],
    ) -> std::cmp::Ordering {
        match self
            .bdd
            .passive_var_impact(rhs.0, interpr)
            .cmp(&self.bdd.passive_var_impact(lhs.0, interpr))
        {
            std::cmp::Ordering::Equal => match self
                .bdd
                .active_var_impact(lhs.0, interpr)
                .cmp(&self.bdd.active_var_impact(rhs.0, interpr))
            {
                std::cmp::Ordering::Equal => self
                    .bdd
                    .paths(lhs.1, true)
                    .minimum()
                    .cmp(&self.bdd.paths(rhs.1, true).minimum()),
                value => value,
            },
            value => value,
        }
    }

    fn heu_min_paths_max_imp(
        &self,
        lhs: (Var, Term),
        rhs: (Var, Term),
        interpr: &[Term],
    ) -> std::cmp::Ordering {
        match self
            .bdd
            .paths(lhs.1, true)
            .minimum()
            .cmp(&self.bdd.paths(rhs.1, true).minimum())
        {
            std::cmp::Ordering::Equal => self
                .bdd
                .passive_var_impact(rhs.0, interpr)
                .cmp(&self.bdd.passive_var_impact(lhs.0, interpr)),

            value => value,
        }
    }

    fn two_val_model_counts_logic<H>(
        &mut self,
        interpr: &[Term],
        will_be: &[Term],
        depth: usize,
        heuristic: H,
    ) -> Vec<Vec<Term>>
    where
        H: Fn(&Self, (Var, Term), (Var, Term), &[Term]) -> std::cmp::Ordering + Copy,
    {
        log::debug!("two_val_model_recursion_depth: {}/{}", depth, interpr.len());
        if let Some((idx, ac)) = interpr
            .iter()
            .enumerate()
            .filter(|(idx, val)| !(val.is_truth_value() || will_be[*idx].is_truth_value()))
            .min_by(|(idx_a, val_a), (idx_b, val_b)| {
                heuristic(
                    self,
                    (Var(*idx_a), **val_a),
                    (Var(*idx_b), **val_b),
                    interpr,
                )
            })
        {
            let mut result = Vec::new();
            let check_models = !self.bdd.paths(*ac, true).more_models();
            log::trace!(
                "Identified Var({}) with ac {:?} to be {}",
                idx,
                ac,
                check_models
            );
            let _ = self // return value can be ignored, but must be catched
                .bdd
                .interpretations(*ac, check_models, Var(idx), &[], &[])
                .iter()
                .try_for_each(|(negative, positive)| {
                    let mut new_int = interpr.to_vec();
                    let res = negative
                        .iter()
                        .try_for_each(|var| {
                            if new_int[var.value()].is_true() || will_be[var.value()] == Term::TOP {
                                return Err(());
                            }
                            new_int[var.value()] = Term::BOT;
                            Ok(())
                        })
                        .and(positive.iter().try_for_each(|var| {
                            if (new_int[var.value()].is_truth_value()
                                && !new_int[var.value()].is_true())
                                || will_be[var.value()] == Term::BOT
                            {
                                return Err(());
                            }
                            new_int[var.value()] = Term::TOP;
                            Ok(())
                        }));
                    if res.is_ok() {
                        new_int[idx] = if check_models { Term::TOP } else { Term::BOT };
                        let upd_int = self.update_interpretation_fixpoint(&new_int);
                        if self.check_consistency(&upd_int, will_be) {
                            result.append(&mut self.two_val_model_counts_logic(
                                &upd_int,
                                will_be,
                                depth + 1,
                                heuristic,
                            ));
                        }
                    }
                    res
                });
            log::trace!("results found so far:{}", result.len());
            // checked one alternative, we can now conclude that only the other option may work
            log::debug!("checked one alternative, concluding the other value");
            let new_int = interpr
                .iter()
                .map(|tree| self.bdd.restrict(*tree, Var(idx), !check_models))
                .collect::<Vec<Term>>();
            let mut upd_int = self.update_interpretation_fixpoint(&new_int);

            log::trace!("\nnew_int {new_int:?}\nupd_int {upd_int:?}");
            if new_int[idx].no_inf_inconsistency(&upd_int[idx]) {
                upd_int[idx] = if check_models { Term::BOT } else { Term::TOP };
                if new_int[idx].no_inf_inconsistency(&upd_int[idx]) {
                    let mut must_be_new = will_be.to_vec();
                    must_be_new[idx] = new_int[idx];
                    result.append(&mut self.two_val_model_counts_logic(
                        &upd_int,
                        &must_be_new,
                        depth + 1,
                        heuristic,
                    ));
                }
            }
            result
        } else {
            // filter has created empty iterator
            let concluded = interpr
                .iter()
                .enumerate()
                .map(|(idx, val)| {
                    if !val.is_truth_value() {
                        will_be[idx]
                    } else {
                        *val
                    }
                })
                .collect::<Vec<Term>>();
            let ac = self.ac.clone();
            let result = self.apply_interpretation(&ac, &concluded);
            if self.check_consistency(&result, &concluded) {
                vec![result]
            } else {
                vec![interpr.to_vec()]
            }
        }
    }

    fn update_interpretation_fixpoint(&mut self, interpretation: &[Term]) -> Vec<Term> {
        let mut cur_int = interpretation.to_vec();
        loop {
            let new_int = self.update_interpretation(interpretation);
            if cur_int == new_int {
                return cur_int;
            } else {
                cur_int = new_int;
            }
        }
    }

    /// Constructs the fixpoint of the given interpretation with respect to the ADF.
    /// sets _update_ to [`true`] if the value has been updated and to [`false`] otherwise.
    fn update_interpretation_fixpoint_upd(
        &mut self,
        interpretation: &[Term],
        update: &mut bool,
    ) -> Vec<Term> {
        let mut cur_int = interpretation.to_vec();
        *update = false;
        loop {
            let new_int = self.update_interpretation(interpretation);
            if cur_int == new_int {
                return cur_int;
            } else {
                cur_int = new_int;
                *update = true;
            }
        }
    }

    fn update_interpretation(&mut self, interpretation: &[Term]) -> Vec<Term> {
        self.apply_interpretation(interpretation, interpretation)
    }

    fn apply_interpretation(&mut self, ac: &[Term], interpretation: &[Term]) -> Vec<Term> {
        ac.iter()
            .map(|ac| {
                interpretation
                    .iter()
                    .enumerate()
                    .fold(*ac, |acc, (idx, val)| {
                        if val.is_truth_value() {
                            self.bdd.restrict(acc, Var(idx), val.is_true())
                        } else {
                            acc
                        }
                    })
            })
            .collect::<Vec<Term>>()
    }

    fn check_consistency(&mut self, interpretation: &[Term], will_be: &[Term]) -> bool {
        interpretation
            .iter()
            .zip(will_be.iter())
            .all(|(int, wb)| wb.no_inf_inconsistency(int))
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

    /// Creates a [PrintableInterpretation] for output purposes.
    pub fn print_interpretation<'a, 'b>(
        &'a self,
        interpretation: &'b [Term],
    ) -> PrintableInterpretation<'b>
    where
        'a: 'b,
    {
        PrintableInterpretation::new(interpretation, &self.ordering)
    }

    /// Creates a [PrintDictionary] for output purposes.
    pub fn print_dictionary(&self) -> PrintDictionary {
        PrintDictionary::new(&self.ordering)
    }

    /// Fixes the bdd after an import with serde.
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

                let fc = match mcs.models > 2 {
                    true => 2 * n_vdps(*t),
                    _ => 0,
                };
                let cfc = match mcs.cmodels > 2 {
                    true => 2 * n_vdps(*t),
                    _ => 0,
                };
                (mcs, (cfc, fc))
            })
            .collect::<Vec<_>>()
    }

    /// Computes the stable extensions of a given [`Adf`], using the [`NoGood`]-learner.
    pub fn stable_nogood<'a, 'c>(
        &'a mut self,
        heuristic: Heuristic,
    ) -> impl Iterator<Item = Vec<Term>> + 'c
    where
        'a: 'c,
    {
        let grounded = self.grounded();
        let heu = heuristic.get_heuristic();
        let (s, r) = crossbeam_channel::unbounded::<Vec<Term>>();
        self.stable_nogood_get_vec(&grounded, heu, s, r).into_iter()
    }

    /// Computes the stable extension of a given [`Adf`], using the [`NoGood`]-learner.
    /// Needs a [`Sender`][crossbeam_channel::Sender<Vec<crate::datatypes::Term>>] where the results of the computation can be put to.
    pub fn stable_nogood_channel(
        &mut self,
        heuristic: Heuristic,
        sender: crossbeam_channel::Sender<Vec<Term>>,
    ) {
        let grounded = self.grounded();
        self.stable_nogood_internal(&grounded, heuristic.get_heuristic(), sender);
    }

    fn stable_nogood_get_vec<H>(
        &mut self,
        interpretation: &[Term],
        heuristic: H,
        s: crossbeam_channel::Sender<Vec<Term>>,
        r: crossbeam_channel::Receiver<Vec<Term>>,
    ) -> Vec<Vec<Term>>
    where
        H: Fn(&Self, &[Term]) -> Option<(Var, Term)>,
    {
        self.stable_nogood_internal(interpretation, heuristic, s);
        r.iter().collect()
    }

    fn stable_nogood_internal<H>(
        &mut self,
        interpretation: &[Term],
        heuristic: H,
        s: crossbeam_channel::Sender<Vec<Term>>,
    ) where
        H: Fn(&Self, &[Term]) -> Option<(Var, Term)>,
    {
        let mut cur_interpr = interpretation.to_vec();
        let mut ng_store = NoGoodStore::new(
            self.ac
                .len()
                .try_into()
                .expect("Expecting only u32 many statements"),
        );
        let mut stack: Vec<(bool, NoGood)> = Vec::new();
        let mut interpr_history: Vec<Vec<Term>> = Vec::new();
        let mut backtrack = false;
        let mut update_ng;
        let mut update_fp = false;
        let mut choice = false;

        log::debug!("start learning loop");
        loop {
            log::trace!("interpr: {:?}", cur_interpr);
            log::trace!("choice: {}", choice);
            if choice {
                choice = false;
                if let Some((var, term)) = heuristic(&*self, &cur_interpr) {
                    log::trace!("choose {}->{}", var, term.is_true());
                    interpr_history.push(cur_interpr.to_vec());
                    cur_interpr[var.value()] = term;
                    stack.push((true, cur_interpr.as_slice().into()));
                } else {
                    backtrack = true;
                }
            }
            update_ng = true;
            log::trace!("backtrack: {}", backtrack);
            if backtrack {
                backtrack = false;
                if stack.is_empty() {
                    break;
                }
                while let Some((choice, ng)) = stack.pop() {
                    log::trace!("adding ng: {:?}", ng);
                    ng_store.add_ng(ng);

                    if choice {
                        cur_interpr = interpr_history.pop().expect("both stacks (interpr_history and `stack`) should always be synchronous");
                        log::trace!(
                            "choice found, reverting interpretation to {:?}",
                            cur_interpr
                        );
                        break;
                    }
                }
            }
            match ng_store.conclusion_closure(&cur_interpr) {
                crate::nogoods::ClosureResult::Update(new_int) => {
                    cur_interpr = new_int;
                    log::trace!("ng update: {:?}", cur_interpr);
                    stack.push((false, cur_interpr.as_slice().into()));
                }
                crate::nogoods::ClosureResult::NoUpdate => {
                    log::trace!("no update");
                    update_ng = false;
                }
                crate::nogoods::ClosureResult::Inconsistent => {
                    log::trace!("inconsistency");
                    backtrack = true;
                    continue;
                }
            }

            let ac_consistent_interpr = self.apply_interpretation(&self.ac.clone(), &cur_interpr);
            log::trace!(
                "checking consistency of {:?} against {:?}",
                ac_consistent_interpr,
                cur_interpr
            );
            if cur_interpr
                .iter()
                .zip(ac_consistent_interpr.iter())
                .any(|(cur, ac)| {
                    cur.is_truth_value() && ac.is_truth_value() && cur.is_true() != ac.is_true()
                })
            {
                log::trace!("ac_inconsistency");
                backtrack = true;
                continue;
            }

            cur_interpr = self.update_interpretation_fixpoint_upd(&cur_interpr, &mut update_fp);
            if update_fp {
                log::trace!("fixpount updated");
                //stack.push((false, cur_interpr.as_slice().into()));
            } else if !update_ng {
                // No updates done this loop
                if !self.is_two_valued(&cur_interpr) {
                    choice = true;
                } else if self.stability_check(&cur_interpr) {
                    // stable model found
                    stack.push((false, cur_interpr.as_slice().into()));
                    s.send(cur_interpr.clone())
                        .expect("Sender should accept results");
                    backtrack = true;
                } else {
                    // not stable
                    log::trace!("2 val not stable");
                    stack.push((false, cur_interpr.as_slice().into()));
                    backtrack = true;
                }
            }
        }
        log::info!("{ng_store}");
        log::debug!("{:?}", ng_store);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crossbeam_channel::unbounded;
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
    fn stable_w_counts() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);

        let mut stable = adf.stable_count_optimisation_heu_a();
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
        let mut stable = adf.stable_count_optimisation_heu_a();

        assert_eq!(stable.next(), Some(vec![Term::BOT, Term::TOP]));
        assert_eq!(stable.next(), Some(vec![Term::TOP, Term::BOT]));
        assert_eq!(stable.next(), None);

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,b).ac(b,a).").unwrap();
        let mut adf = Adf::from_parser(&parser);

        assert_eq!(
            adf.stable_count_optimisation_heu_a().collect::<Vec<_>>(),
            vec![vec![Term::BOT, Term::BOT]]
        );

        assert_eq!(
            adf.stable_count_optimisation_heu_b().collect::<Vec<_>>(),
            vec![vec![Term::BOT, Term::BOT]]
        );

        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,neg(a)).ac(b,a).").unwrap();
        let mut adf = Adf::from_parser(&parser);
        assert_eq!(adf.stable_count_optimisation_heu_a().next(), None);
        assert_eq!(adf.stable_count_optimisation_heu_b().next(), None);
    }

    #[test]
    fn stable_nogood() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);

        let grounded = adf.grounded();
        let (s, r) = unbounded();
        adf.stable_nogood_internal(&grounded, crate::adf::heuristics::heu_simple, s);

        assert_eq!(
            r.iter().collect::<Vec<_>>(),
            vec![vec![
                Term::TOP,
                Term::BOT,
                Term::BOT,
                Term::TOP,
                Term::BOT,
                Term::TOP
            ]]
        );
        let mut stable_iter = adf.stable_nogood(Heuristic::Simple);
        assert_eq!(
            stable_iter.next(),
            Some(vec![
                Term::TOP,
                Term::BOT,
                Term::BOT,
                Term::TOP,
                Term::BOT,
                Term::TOP
            ])
        );

        assert_eq!(stable_iter.next(), None);
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).ac(a,neg(b)).ac(b,neg(a)).").unwrap();
        let mut adf = Adf::from_parser(&parser);
        let grounded = adf.grounded();
        let (s, r) = unbounded();
        adf.stable_nogood_internal(&grounded, crate::adf::heuristics::heu_simple, s.clone());
        let stable_result = r.try_iter().collect::<Vec<_>>();
        assert_eq!(
            stable_result,
            vec![vec![Term(1), Term(0)], vec![Term(0), Term(1)]]
        );

        let stable = adf.stable_nogood(Heuristic::Simple);
        assert_eq!(
            stable.collect::<Vec<_>>(),
            vec![vec![Term(1), Term(0)], vec![Term(0), Term(1)]]
        );

        let stable = adf.stable_nogood(Heuristic::Custom(&|_adf, interpr| {
            for (idx, term) in interpr.iter().enumerate() {
                if !term.is_truth_value() {
                    return Some((Var(idx), Term::BOT));
                }
            }
            None
        }));
        assert_eq!(
            stable.collect::<Vec<_>>(),
            vec![vec![Term(0), Term(1)], vec![Term(1), Term(0)]]
        );

        adf.stable_nogood_channel(Heuristic::default(), s);
        assert_eq!(
            r.iter().collect::<Vec<_>>(),
            vec![vec![Term(1), Term(0)], vec![Term(0), Term(1)]]
        );

        // multi-threaded usage
        let (s, r) = unbounded();
        let solving = std::thread::spawn(move || {
            let parser = AdfParser::default();
            parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
            let mut adf = Adf::from_parser(&parser);
            adf.stable_nogood_channel(Heuristic::MinModMaxVarImpMinPaths, s.clone());
            adf.stable_nogood_channel(Heuristic::MinModMinPathsMaxVarImp, s);
        });

        let mut result_vec = Vec::new();
        while let Ok(result) = r.recv() {
            result_vec.push(result);
        }
        assert_eq!(
            result_vec,
            vec![
                vec![
                    Term::TOP,
                    Term::BOT,
                    Term::BOT,
                    Term::TOP,
                    Term::BOT,
                    Term::TOP
                ],
                vec![
                    Term::TOP,
                    Term::BOT,
                    Term::BOT,
                    Term::TOP,
                    Term::BOT,
                    Term::TOP
                ]
            ]
        );
        solving.join().unwrap();
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

    #[cfg(feature = "adhoccountmodels")]
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
    #[cfg(feature = "adhoccountmodels")]
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
