use assert_cmd::prelude::*; // Add methods on commands
use assert_fs::prelude::*;
use predicates::prelude::*; // Used for writing assertions
use std::process::Command; // Run programs

#[test]
fn arguments() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg("-vvv").arg("--lx").arg("file.txt");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg("-v").arg("--lx").arg("--an").arg("file.txt");
    cmd.assert().failure().stderr(predicate::str::contains(
        "cannot be used with one or more of the other specified arguments",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg("-h");
    cmd.assert().success().stdout(predicate::str::contains(
        "stefan.ellmauthaler@tu-dresden.de",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("adf_bdd "));
    Ok(())
}

#[test]
fn runs_naive() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("input_instance.adf")?;
    file.write_str("s(7).s(4).s(8).s(3).s(5).s(9).s(10).s(1).s(6).s(2).ac(7,or(or(and(7,neg(1)),neg(9)),3)).ac(4,5).ac(8,or(or(8,1),neg(7))).ac(3,or(and(or(6,7),neg(and(6,7))),neg(2))).ac(5,c(f)).ac(9,and(neg(7),2)).ac(10,or(neg(2),6)).ac(1,and(or(or(neg(2),neg(1)),8),7)).ac(6,and(and(neg(2),10),and(or(7,4),neg(and(7,4))))).ac(2,and(and(and(neg(10),3),neg(6)),or(9,1))).")?;
    let wrong_file = assert_fs::NamedTempFile::new("wrong_format.adf")?;
    wrong_file.write_str("s(7).s(4).s(8).s(3).s(5).s(9).s(10).s(1).s(6).s(2).ac(7,or(or(and(7,neg(1)),neg(9)),3)).ac(4,5).ac(8,or(or(8,1),neg(7))).ac(3,or(and(or(6,7),neg(and(6,7))),neg(2))).ac(5,c(f)).ac(9,and(neg(7),2)).ac(10,or(neg(2),6)).ac(1,and(or(or(neg(2),neg(1)),8),7)).ac(6,and(and(neg(2),10),and(or(7,4),neg(and(7,4))))).ac(2,and(and(and(neg(10),3),neg(6)),or(9,1)))).")?;

    let mut cmd = Command::cargo_bin("adf_bdd")?;

    cmd.arg(wrong_file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("code: Eof"));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("-vv")
        .arg("--grd")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(7) F(4) u(8) u(3) F(5) u(9) u(10) u(1) u(6) u(2)",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("-q")
        .arg("--grd")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(7) F(4) u(8) u(3) F(5) u(9) u(10) u(1) u(6) u(2)",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--lx")
        .arg("-v")
        .arg("--grd")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(10) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9)",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--stm")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.env_clear();
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--rust_log")
        .arg("trace")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--rust_log")
        .arg("warn")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    let tempdir = assert_fs::TempDir::new()?;

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--lib")
        .arg("naive")
        .arg("--export")
        .arg(tempdir.path().with_file_name("test.json"));
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--lib")
        .arg("naive")
        .arg("--export")
        .arg(tempdir.path().with_file_name("test.json"));
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(tempdir.path().with_file_name("test.json"))
        .arg("--an")
        .arg("--grd")
        .arg("--import")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--com")
        .arg("--rust_log")
        .arg("warn")
        .arg("--lib")
        .arg("naive");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));
    Ok(())
}

#[test]
fn runs_biodivine() -> Result<(), Box<dyn std::error::Error>> {
    let file = assert_fs::NamedTempFile::new("input_instance.adf")?;
    file.write_str("s(7).s(4).s(8).s(3).s(5).s(9).s(10).s(1).s(6).s(2).ac(7,or(or(and(7,neg(1)),neg(9)),3)).ac(4,5).ac(8,or(or(8,1),neg(7))).ac(3,or(and(or(6,7),neg(and(6,7))),neg(2))).ac(5,c(f)).ac(9,and(neg(7),2)).ac(10,or(neg(2),6)).ac(1,and(or(or(neg(2),neg(1)),8),7)).ac(6,and(and(neg(2),10),and(or(7,4),neg(and(7,4))))).ac(2,and(and(and(neg(10),3),neg(6)),or(9,1))).")?;
    let wrong_file = assert_fs::NamedTempFile::new("wrong_format.adf")?;
    wrong_file.write_str("s(7).s(4).s(8).s(3).s(5).s(9).s(10).s(1).s(6).s(2).ac(7,or(or(and(7,neg(1)),neg(9)),3)).ac(4,5).ac(8,or(or(8,1),neg(7))).ac(3,or(and(or(6,7),neg(and(6,7))),neg(2))).ac(5,c(f)).ac(9,and(neg(7),2)).ac(10,or(neg(2),6)).ac(1,and(or(or(neg(2),neg(1)),8),7)).ac(6,and(and(neg(2),10),and(or(7,4),neg(and(7,4))))).ac(2,and(and(and(neg(10),3),neg(6)),or(9,1)))).")?;

    let mut cmd = Command::cargo_bin("adf_bdd")?;

    cmd.arg(wrong_file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("code: Eof"));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path()).arg("-vv").arg("--grd");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(7) F(4) u(8) u(3) F(5) u(9) u(10) u(1) u(6) u(2)",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path()).arg("-q").arg("--grd");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(7) F(4) u(8) u(3) F(5) u(9) u(10) u(1) u(6) u(2)",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path()).arg("--lx").arg("-v").arg("--grd");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(10) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9)",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path()).arg("--an").arg("--grd").arg("--stm");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.env_clear();
    cmd.arg(file.path()).arg("--an").arg("--grd");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--rust_log")
        .arg("trace");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));

    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--grd")
        .arg("--rust_log")
        .arg("warn");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));
    cmd = Command::cargo_bin("adf_bdd")?;
    cmd.arg(file.path())
        .arg("--an")
        .arg("--com")
        .arg("--rust_log")
        .arg("warn");
    cmd.assert().success().stdout(predicate::str::contains(
        "u(1) u(2) u(3) F(4) F(5) u(6) u(7) u(8) u(9) u(10) \n",
    ));
    Ok(())
}
