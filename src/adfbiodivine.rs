//! This module describes the abstract dialectical framework
//!
//!  - computing interpretations
//!  - computing fixpoints

use crate::{
    datatypes::{
        adf::{
            PrintableInterpretation, ThreeValuedInterpretationsIterator,
            TwoValuedInterpretationsIterator, VarContainer,
        },
        Term,
    },
    parser::AdfParser,
};

use biodivine_lib_bdd::Bdd;

//#[derive(Serialize, Deserialize, Debug)]
/// Representation of an ADF, with an ordering and dictionary of statement <-> number relations, a binary decision diagram, and a list of acceptance functions in Term representation
// pub struct Adf {
//     ordering: VarContainer,
//     bdd: Bdd,
//     ac: Vec<Term>,
// }
#[derive(Debug)]
/// Representation of an ADF, with an ordering and dictionary of statement <-> number relations, a binary decision diagram, and a list of acceptance functions in Term representation - TODO
pub struct Adf {
    ordering: VarContainer,
    ac: Vec<Bdd>,
    vars: Vec<biodivine_lib_bdd::BddVariable>,
}

impl Adf {
    /// Instantiates a new ADF, based on the parser-data
    pub fn from_parser(parser: &AdfParser) -> Self {
        log::info!("[Start] instantiating BDD");
        let mut bdd_var_builder = biodivine_lib_bdd::BddVariableSetBuilder::new();
        let namelist = parser.namelist_rc_refcell().as_ref().borrow().clone();
        let slice_vec: Vec<&str> = namelist.iter().map(<_>::as_ref).collect();
        bdd_var_builder.make_variables(&slice_vec);
        let bdd_variables = bdd_var_builder.build();
        let mut result = Self {
            ordering: VarContainer::from_parser(
                parser.namelist_rc_refcell(),
                parser.dict_rc_refcell(),
            ),
            ac: vec![
                bdd_variables.mk_false();
                parser.namelist_rc_refcell().as_ref().borrow().len()
            ],
            vars: bdd_variables.variables(),
        };
        log::trace!("variable order: {:?}", result.vars);
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
                result.ac[*new_order] = bdd_variables
                    .eval_expression(&parser.ac_at(insert_order).unwrap().to_boolean_expr());
                log::trace!("instantiated {}", result.ac[*new_order]);
            });
        log::info!("[Success] instantiated");
        result
    }
    /// Computes the grounded extension and returns it as a list
    pub fn grounded(&mut self) -> Vec<Term> {
        log::info!("[Start] grounded");
        let ac = &self.ac.clone();
        let result = self
            .grounded_internal(ac)
            .iter()
            .map(|elem| elem.into())
            .collect();
        log::info!("[Done] grounded");
        result
    }

    fn grounded_internal(&mut self, interpretation: &[Bdd]) -> Vec<Bdd> {
        let mut new_interpretation: Vec<Bdd> = interpretation.into();
        loop {
            let mut truth_extention: bool = false;
            let var_list: Vec<(biodivine_lib_bdd::BddVariable, bool)> = self
                .vars
                .iter()
                .enumerate()
                .filter(|(idx, _elem)| new_interpretation[*idx].is_truth_value())
                .map(|(idx, elem)| (*elem, new_interpretation[idx].is_true()))
                .collect();

            log::trace!("var-list: {:?}", var_list);

            for ac in new_interpretation
                .iter_mut()
                .filter(|elem| !elem.is_truth_value())
            {
                log::trace!("checking ac: {}", ac);
                *ac = ac.restrict(&var_list);
                if ac.is_truth_value() {
                    truth_extention = true;
                }
            }
            if !truth_extention {
                break;
            }
        }
        new_interpretation
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

/// Provides Adf-Specific operations on truth valuations
pub trait AdfOperations {
    /// Returns `true` if the BDD is either valid or unsatisfiable
    fn is_truth_value(&self) -> bool;

    /// Compares whether the information between two given BDDs are the same
    fn cmp_information(&self, other: &Self) -> bool;
}

impl AdfOperations for Bdd {
    fn is_truth_value(&self) -> bool {
        self.is_false() || self.is_true()
    }

    fn cmp_information(&self, other: &Self) -> bool {
        self.is_truth_value() == other.is_truth_value() && self.is_true() == other.is_true()
    }
}

/// Implementations of the restrict-operations on BDDs
pub trait BddRestrict {
    /// Provides an implementation of the restrict-operation on BDDs for one variable
    fn var_restrict(&self, variable: biodivine_lib_bdd::BddVariable, value: bool) -> Self;
    /// Provides an implementation of the restrict-operation on a set of variables
    fn restrict(&self, variables: &[(biodivine_lib_bdd::BddVariable, bool)]) -> Self;
}

impl BddRestrict for Bdd {
    fn var_restrict(&self, variable: biodivine_lib_bdd::BddVariable, value: bool) -> Bdd {
        self.var_select(variable, value).var_project(variable)
    }

    fn restrict(&self, variables: &[(biodivine_lib_bdd::BddVariable, bool)]) -> Bdd {
        let mut variablelist: Vec<biodivine_lib_bdd::BddVariable> = Vec::new();
        variables
            .iter()
            .for_each(|(var, _val)| variablelist.push(*var));
        self.select(variables).project(&variablelist)
    }
}

impl ThreeValuedInterpretationsIterator {
    fn from_bdd(bdd: &[Bdd]) -> Self {
        let terms = bdd.iter().map(|value| value.into()).collect::<Vec<_>>();
        Self::new(&terms)
    }
}

impl TwoValuedInterpretationsIterator {
    fn from_bdd(bdd: &[Bdd]) -> Self {
        let terms = bdd.iter().map(|value| value.into()).collect::<Vec<_>>();
        Self::new(&terms)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use biodivine_lib_bdd::BddVariable;
    use test_log::test;
    #[test]
    fn grounded() {
        let parser = AdfParser::default();
        parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
        let mut adf = Adf::from_parser(&parser);

        let xor = adf.ac[5].clone();

        assert!(xor
            .var_restrict(adf.vars[4], false)
            .var_restrict(adf.vars[0], true)
            .var_restrict(adf.vars[1], true)
            .var_restrict(adf.vars[2], false)
            .is_true());
        let result = adf.grounded();

        assert_eq!(
            result,
            vec![
                Term::TOP,
                Term::UND,
                Term::UND,
                Term::UND,
                Term::BOT,
                Term::TOP
            ]
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
        assert_eq!(
            result,
            vec![Term::TOP, Term::TOP, Term::TOP, Term::BOT, Term::BOT]
        );
    }
}
