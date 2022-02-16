use super::adf::Adf as NaiveAdf;
use super::adfbiodivine::*;
use super::datatypes::*;
use super::parser::*;
use test_log::test;

#[test]
fn adf_biodivine_cmp_1() {
    let parser = AdfParser::default();
    parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
    let adf = Adf::from_parser(&parser);
    let mut naive_adf = NaiveAdf::from_biodivine(&adf);

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

    let naive_result = naive_adf.grounded();
    assert_eq!(
        format!("{}", adf.print_interpretation(&adf.grounded())),
        format!("{}", naive_adf.print_interpretation(&naive_result))
    );

    let printer = naive_adf.print_dictionary();
    let mut str1 = String::new();
    let mut str2 = String::new();
    let mut str3 = String::new();
    for model in adf.stable() {
        str1 = format!("{}{}", str1, adf.print_interpretation(&model));
    }
    for model in naive_adf.stable() {
        str2 = format!("{}{}", str2, printer.print_interpretation(&model));
    }

    for model in adf.hybrid_step().stable() {
        str3 = format!("{}{}", str3, printer.print_interpretation(&model));
    }

    assert_eq!(str1, str2);
    assert_eq!(str1, str3);

    let mut str1 = String::new();
    let mut str2 = String::new();
    let mut str3 = String::new();
    for model in adf.complete() {
        str1 = format!("{}{}", str1, adf.print_interpretation(&model));
    }
    for model in naive_adf.complete() {
        str2 = format!("{}{}", str2, printer.print_interpretation(&model));
    }

    for model in adf.hybrid_step().complete() {
        str3 = format!("{}{}", str3, printer.print_interpretation(&model));
    }

    assert_eq!(str1, str2);
    assert_eq!(str1, str3);

    let parser = AdfParser::default();
    parser.parse()(
        "s(a).s(b).s(c).s(d).s(e).ac(a,c(v)).ac(b,a).ac(c,b).ac(d,neg(c)).ac(e,and(a,d)).",
    )
    .unwrap();
    let adf = Adf::from_parser(&parser);
    let mut naive_adf = NaiveAdf::from_biodivine(&adf);
    let result = adf.grounded();
    let naive_result = naive_adf.grounded();
    assert_eq!(
        result,
        vec![Term::TOP, Term::TOP, Term::TOP, Term::BOT, Term::BOT]
    );

    assert_eq!(
        format!("{}", adf.print_interpretation(&result)),
        format!("{}", naive_adf.print_interpretation(&naive_result))
    );
}

#[test]
fn adf_biodivine_cmp_2() {
    let parser = AdfParser::default();
    parser.parse()(
        "s(a).s(b).s(c).s(d).s(e).ac(a,c(v)).ac(b,a).ac(c,b).ac(d,neg(c)).ac(e,and(a,d)).",
    )
    .unwrap();
    let adf = Adf::from_parser(&parser);
    let mut naive_adf = NaiveAdf::from_biodivine(&adf);
    let result = adf.grounded();
    let naive_result = naive_adf.grounded();
    assert_eq!(
        result,
        vec![Term::TOP, Term::TOP, Term::TOP, Term::BOT, Term::BOT]
    );

    assert_eq!(
        format!("{}", adf.print_interpretation(&result)),
        format!("{}", naive_adf.print_interpretation(&naive_result))
    );

    let printer = naive_adf.print_dictionary();
    let mut str1 = String::new();
    let mut str2 = String::new();
    let mut str3 = String::new();
    for model in adf.stable() {
        str1 = format!("{}{}", str1, adf.print_interpretation(&model));
    }
    for model in naive_adf.stable() {
        str2 = format!("{}{}", str2, printer.print_interpretation(&model));
    }

    for model in adf.hybrid_step().stable() {
        str3 = format!("{}{}", str3, printer.print_interpretation(&model));
    }

    assert_eq!(str1, str2);
    assert_eq!(str1, str3);

    let mut str1 = String::new();
    let mut str2 = String::new();
    let mut str3 = String::new();
    for model in adf.complete() {
        str1 = format!("{}{}", str1, adf.print_interpretation(&model));
    }
    for model in naive_adf.complete() {
        str2 = format!("{}{}", str2, printer.print_interpretation(&model));
    }

    for model in adf.hybrid_step().complete() {
        str3 = format!("{}{}", str3, printer.print_interpretation(&model));
    }

    assert_eq!(str1, str2);
    assert_eq!(str1, str3);
}

#[test]
fn stable_variants_cmp() {
    let parser = AdfParser::default();
    parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
        .unwrap();
    let adf = Adf::from_parser_with_stm_rewrite(&parser);
    let mut naive_adf = NaiveAdf::from_biodivine(&adf);

    let mut stable_naive: Vec<Vec<Term>> = naive_adf.stable().collect();
    let mut stable_pre: Vec<Vec<Term>> = naive_adf.stable_with_prefilter().collect();
    let mut stable_v2 = adf.stable_bdd_representation();
    let mut stable_v2_hybrid = naive_adf.stable_bdd_representation(&adf);

    stable_naive.sort();
    stable_pre.sort();
    stable_v2.sort();
    stable_v2_hybrid.sort();

    assert_eq!(stable_naive, stable_v2);
    assert_eq!(stable_v2, stable_v2_hybrid);
    assert_eq!(stable_pre, stable_v2);
}
