use adf_bdd::{adf::Adf, parser::AdfParser};
use test_generator::test_resources;
use test_log::test;

//#[test_resources("res/adf-instances/instances/*.adf")]
fn compute_grounded(resource: &str) {
    let grounded = &[
        "res/adf-instances/grounded-interpretations/",
        &resource[28..resource.len() - 8],
        ".apx.adf-grounded.txt",
    ]
    .concat();
    log::debug!("Grounded: {}", grounded);
    let parser = AdfParser::default();
    let expected_result = std::fs::read_to_string(grounded);
    assert!(expected_result.is_ok());
    let input = std::fs::read_to_string(resource).unwrap();
    parser.parse()(&input).unwrap();
    parser.varsort_alphanum();
    let mut adf = Adf::from_parser(&parser);
    let grounded = adf.grounded();
    assert_eq!(
        format!("{}", adf.print_interpretation(&grounded)),
        expected_result.unwrap()
    );
}
