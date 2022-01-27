//! Buildscript to automatically generate test-cases
//! Idea found at https://blog.cyplo.dev/posts/2018/12/generate-rust-tests-from-data
use std::env;
use std::fs::read_dir;
use std::fs::DirEntry;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// build script's entry point
fn main() {
    gen_tests();
}

fn gen_tests() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let destination = Path::new(&out_dir).join("tests.rs");
    let mut test_file = File::create(&destination).unwrap();

    if let Ok(test_data_directory) = read_dir("./res/adf-instances/instances/") {
        // write test file header, put `use`, `const` etc there
        write_header(&mut test_file);

        for file in test_data_directory {
            write_test(&mut test_file, &file.unwrap());
        }
    }
}

fn write_test(test_file: &mut File, file: &DirEntry) {
    let file = file.path().canonicalize().unwrap();
    let path = file.display();
    let test_name = format!("{}", file.file_name().unwrap().to_string_lossy())
        .replace(".", "_")
        .replace("-", "_")
        .replace("@", "at")
        .to_lowercase();
    let grounded_name = format!(
        "{}-grounded.txt",
        file.as_path().as_os_str().to_string_lossy()
    )
    .replace(
        "adf-instances/instances",
        "adf-instances/grounded-interpretations",
    );

    write!(
        test_file,
        include_str!("./tests/test_template"),
        name = test_name,
        path = path,
        grounded = grounded_name
    )
    .unwrap();
}

fn write_header(test_file: &mut File) {
    write!(
        test_file,
        r#"
use adf_bdd::adfbiodivine::Adf;
use adf_bdd::parser::AdfParser;
use test_log::test;
"#
    )
    .unwrap();
}
