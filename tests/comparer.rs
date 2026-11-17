use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn temp_file(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ypbank-comparer-{}-{name}", std::process::id()))
}

fn run_comparer(left: &Path, right: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_comparer"))
        .args([
            "--left",
            left.to_str().unwrap(),
            "--left-format",
            "csv",
            "--right",
            right.to_str().unwrap(),
            "--right-format",
            "csv",
        ])
        .output()
        .unwrap()
}

#[test]
fn equal_files_print_equal() {
    let left = temp_file("equal-left.csv");
    let right = temp_file("equal-right.csv");
    let data = "date,category,kind,amount\n2026-04-01,salary,income,120000\n";
    fs::write(&left, data).unwrap();
    fs::write(&right, data).unwrap();

    let output = run_comparer(&left, &right);

    fs::remove_file(&left).unwrap();
    fs::remove_file(&right).unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "files are equal"
    );
}

#[test]
fn different_amount_reports_position_and_values() {
    let left = temp_file("diff-left.csv");
    let right = temp_file("diff-right.csv");
    fs::write(
        &left,
        "date,category,kind,amount\n2026-04-01,salary,income,120000\n2026-04-02,rent,expense,40000\n",
    )
    .unwrap();
    fs::write(
        &right,
        "date,category,kind,amount\n2026-04-01,salary,income,120000\n2026-04-02,rent,expense,7000\n",
    )
    .unwrap();

    let output = run_comparer(&left, &right);

    fs::remove_file(&left).unwrap();
    fs::remove_file(&right).unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "transactions differ at position 2: amount: 40000 != 7000"
    );
}

#[test]
fn extra_record_reports_position() {
    let left = temp_file("extra-left.csv");
    let right = temp_file("extra-right.csv");
    fs::write(
        &left,
        "date,category,kind,amount\n2026-04-01,salary,income,120000\n2026-04-02,rent,expense,40000\n",
    )
    .unwrap();
    fs::write(
        &right,
        "date,category,kind,amount\n2026-04-01,salary,income,120000\n",
    )
    .unwrap();

    let output = run_comparer(&left, &right);

    fs::remove_file(&left).unwrap();
    fs::remove_file(&right).unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "extra transaction at position 2 in the left list"
    );
}
