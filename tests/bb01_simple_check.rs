use std::process::Command;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

fn search(state_num: &str, max_steps: &str) -> (String, String, String) {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("bb01-search-{}-{unique}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_bb01_simple_check"))
        .args([state_num, "--max-steps", max_steps])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "CSV search must not print debug output"
    );
    assert!(output.stderr.is_empty());
    let results = dir.join(format!("results/bb_quest_{state_num}/01_simple_search"));
    let decided = fs::read_to_string(results.join("decided.csv")).unwrap();
    let unresolved = fs::read_to_string(results.join("unresolved.csv")).unwrap();
    let skipped = fs::read_to_string(results.join("skipped.csv")).unwrap();
    fs::remove_dir_all(&dir).unwrap();
    (decided, unresolved, skipped)
}

fn run(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bb01_simple_check"))
        .args(args)
        .output()
        .expect("failed to run bb01_simple_check");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn left_moves_can_cross_the_origin() {
    let output = run(&["0LA 0RH", "--max-steps", "2"]);
    assert!(output.contains("min:-2 ... [0]00... max:0"));
}

#[test]
fn halt_instruction_writes_and_moves() {
    let output = run(&["1RH 0RH", "--max-steps", "1"]);
    assert!(output.contains("=> min:0 ... 1[0]... max:1"));
    assert!(!output.contains("step: 2"));
}

#[test]
fn exhausted_search_terminates_after_all_root_candidates() {
    let (decided, unresolved, skipped) = search("1", "0");
    assert_eq!(decided, "pattern,result,steps\n");
    let patterns: Vec<_> = unresolved.lines().collect();
    assert_eq!(
        patterns,
        [
            "pattern,result,steps",
            "0RA 0RH,unresolved,0",
            "1RA 0RH,unresolved,0",
        ]
    );
    assert_eq!(
        skipped.lines().collect::<Vec<_>>(),
        ["pattern,result", "0LA 0RH,skipped", "1LA 0RH,skipped",]
    );
}

#[test]
fn search_reaches_later_leaf_candidates_and_counts_the_halt_step() {
    let (decided, unresolved, skipped) = search("2", "6");
    assert!(skipped.lines().count() > 1);
    assert!(decided.lines().any(|line| line == "1RB 1LB 1LA 0RH,halt,6"));
    assert_eq!(decided.lines().next(), Some("pattern,result,steps"));
    assert_eq!(unresolved.lines().next(), Some("pattern,result,steps"));
    for (csv, expected_result) in [(&decided, "halt"), (&unresolved, "unresolved")] {
        for line in csv.lines().skip(1) {
            let fields: Vec<_> = line.split(',').collect();
            assert_eq!(fields.len(), 3);
            assert_eq!(fields[0].split_whitespace().count(), 4);
            assert_eq!(fields[1], expected_result);
            let steps = fields[2].parse::<usize>().unwrap();
            if expected_result == "halt" {
                assert!((1..=6).contains(&steps));
            } else {
                assert_eq!(steps, 6);
            }
        }
    }
}
