use std::{fs, process::Command};

#[test]
fn csv_reads_named_column_and_writes_unresolved_rows() {
    let root = std::env::temp_dir().join(format!("yanbb-csv-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let input = root.join("input.csv");
    fs::write(
        &input,
        "note,pattern\n\"comma, quoted\",0RA 0RA\nsecond,0LA 0LA\n",
    )
    .unwrap();
    let output = root.join("out");
    let result = Command::new(env!("CARGO_BIN_EXE_bb03_linear_growth"))
        .args(["--csv"])
        .arg(&input)
        .arg("--output-dir")
        .arg(&output)
        .args(["--max-steps", "0", "--debug", "false"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let mut rows = csv::Reader::from_path(output.join("unresolved.csv")).unwrap();
    let rows = rows.records().collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(&rows[0][0], "0RA 0RA");
    assert_eq!(&rows[1][1], "unresolved");
    assert_eq!(
        csv::Reader::from_path(output.join("decided.csv"))
            .unwrap()
            .records()
            .count(),
        0
    );
    // Reject reuse of an output as input before it can be truncated.
    let saved = fs::read(output.join("unresolved.csv")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_bb03_linear_growth"))
        .arg("--csv")
        .arg(output.join("unresolved.csv"))
        .arg("--output-dir")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read(output.join("unresolved.csv")).unwrap(), saved);
    fs::remove_dir_all(root).unwrap();
}
