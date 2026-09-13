use std::ops::Range;

use clap::Parser;
use itertools::Itertools;
use std::fmt::Debug;
use yan_busy_beaver::algorithm::arithmetic_difference::find_sequence_with_arithmetic_differences;
use yan_busy_beaver::algorithm::arithmetic_difference::get_diff_for_sequence_with_arithmetic_differences;
use yan_busy_beaver::algorithm::arithmetic_difference::next_for_sequence_with_arithmetic_differences;
use yan_busy_beaver::base::HALT_STATE;
use yan_busy_beaver::base::Rule;
use yan_busy_beaver::base::RuleTable;
use yan_busy_beaver::base::State;
use yan_busy_beaver::base::Symbol;
use yan_busy_beaver::base::dir_to_str;
use yan_busy_beaver::base::symbol_to_str;
use yan_busy_beaver::block::Block;
use yan_busy_beaver::block::RepeatedSymbolsLinearBlock;
use yan_busy_beaver::block::SymbolsBlock;
use yan_busy_beaver::counter::CounterExpr;
use yan_busy_beaver::counter::FixedLinearExpr;
use yan_busy_beaver::debug_println;
use yan_busy_beaver::interpreter::Context;
use yan_busy_beaver::interpreter::process_with_debug;
use yan_busy_beaver::tape::Tape;
use yan_busy_beaver::transition::RepeatedRulesLinearTransition;
use yan_busy_beaver::transition::RulesTransition;
use yan_busy_beaver::transition::Transition;
#[derive(Debug, Clone, PartialEq, Eq)]
struct RawExecutionRecord {
    step: i64,
    rule: Rule,
    position: i64,
    max_position: i64,
    min_position: i64,
    data: Vec<Symbol>,
}

#[derive(Parser)]
struct Args {
    #[arg(required_unless_present = "csv", conflicts_with = "csv")]
    pattern_or_state_num: Option<String>,
    /// CSV input with a pattern column.
    #[arg(long)]
    csv: Option<std::path::PathBuf>,
    #[arg(long, requires = "csv")]
    output_dir: Option<std::path::PathBuf>,
    /// Maximum steps (default: 100 for a pattern, 1000 for CSV search).
    #[clap(long)]
    max_steps: Option<usize>,
    /// Show execution, tape block, and comparison diagnostics.
    #[arg(long, action = clap::ArgAction::Set)]
    debug: Option<bool>,
    /// Pattern mode only: auto = try the pattern, then its L/R mirror; off = pattern only; only = mirror only.
    #[arg(long, value_enum, default_value_t = ReverseMode::Auto, conflicts_with = "csv")]
    reverse: ReverseMode,
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
enum ReverseMode {
    Auto,
    Off,
    Only,
}

fn reverse_pattern(pattern: &str) -> String {
    pattern
        .chars()
        .map(|c| match c {
            'L' => 'R',
            'R' => 'L',
            c => c,
        })
        .collect()
}

/// Run `check` on the pattern and/or its L/R mirror according to `mode`.
fn check_with_reverse(pattern: &str, max_steps: usize, debug: bool, mode: ReverseMode) -> bool {
    if mode != ReverseMode::Only {
        debug_println!(
            debug,
            "
{}
PATTERN {} max_steps {}",
            "*".repeat(80),
            pattern,
            max_steps
        );
        if check(pattern, max_steps, debug).is_some() {
            return true;
        }
        if mode == ReverseMode::Off {
            return false;
        }
    }
    let reversed = reverse_pattern(pattern);
    debug_println!(
        debug,
        "
{}
REVERSED PATTERN {} (from {}) max_steps {}",
        "*".repeat(80),
        reversed,
        pattern,
        max_steps
    );
    check(&reversed, max_steps, debug).is_some()
}

fn check(pattern: &str, max_steps: usize, debug: bool) -> Option<()> {
    // 実行履歴の取得
    let execution_records = get_execution_records(pattern, max_steps);
    if debug {
        print_execution_records(&execution_records);
    }

    // minポジション変化点の取得(各Ruleグループごと)
    if debug {
        print_min_changed_sequences(&execution_records);
    }
    let min_changed_sequences =
        find_min_changed_sequences_by_rule(&execution_records, max_steps, debug);
    if min_changed_sequences.is_none() {
        debug_println!(debug, "  => no candidate survived: unresolved");
    }
    let min_changed_sequences = min_changed_sequences?;

    // 各minポジション変化点のシーケンスに対して繰り返しブロックを探索
    for (index, min_changed_seq) in min_changed_sequences.iter().enumerate() {
        debug_println!(
            debug,
            "
{}
MIN-CHANGED SEQUENCE {} steps(1-based) {:?}",
            "#".repeat(80),
            index + 1,
            min_changed_seq.iter().map(|s| s + 1).collect::<Vec<_>>()
        );
        if check_min_changed_sequences(min_changed_seq, &execution_records, debug).is_some() {
            return Some(());
        }
    }
    None
}

/// Print every executed step in the same layout as `print_execution_cycles`,
/// marking the steps where min_position moved.
fn print_execution_records(records: &[RawExecutionRecord]) {
    use yan_busy_beaver::base::state_to_str;

    let Some(last_record) = records.last() else {
        return;
    };
    let display_min = last_record.min_position;
    let display_max = last_record.max_position;

    println!(
        "
{}",
        "=".repeat(80)
    );
    println!("EXECUTION   step is 1-based; *symbol is the head; ---- marks a min_position change");
    println!("tape display range: {display_min}..={display_max}");
    for (index, record) in records.iter().enumerate() {
        let min_changed = index > 0 && records[index - 1].min_position != record.min_position;
        if min_changed {
            println!(
                "{} min {} -> {}",
                "-".repeat(60),
                records[index - 1].min_position,
                record.min_position
            );
        }
        println!(
            "s:{:3} {}{} {}{}{}    {}",
            index + 1,
            state_to_str(record.rule.current_state),
            symbol_to_str(record.rule.read_symbol),
            symbol_to_str(record.rule.instruction.write_symbol),
            dir_to_str(record.rule.instruction.dir),
            state_to_str(record.rule.instruction.next_state),
            format_record_tape(record, display_min, display_max),
        );
    }
}

/// Print where min_position moved, grouped by rule, and which groups formed an
/// arithmetic sequence that survives until max_steps.
fn print_min_changed_sequences(records: &[RawExecutionRecord]) {
    use yan_busy_beaver::base::state_to_str;

    println!(
        "
{}",
        "=".repeat(80)
    );
    println!("MIN-CHANGED STEPS (1-based) grouped by rule");
    let mut by_rule: Vec<(Rule, Vec<i64>)> = vec![];
    for w in records.windows(2) {
        if w[0].min_position != w[1].min_position {
            let step = w[1].step + 1;
            match by_rule.iter_mut().find(|(rule, _)| *rule == w[1].rule) {
                Some((_, steps)) => steps.push(step),
                None => by_rule.push((w[1].rule, vec![step])),
            }
        }
    }
    if by_rule.is_empty() {
        println!("  (min_position never changed)");
    }
    for (rule, steps) in &by_rule {
        let diffs = steps.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
        let diffs2 = diffs.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
        println!(
            "  {}{} {}{}{}  steps {:?}  diffs {:?}  diffs2 {:?}",
            state_to_str(rule.current_state),
            symbol_to_str(rule.read_symbol),
            symbol_to_str(rule.instruction.write_symbol),
            dir_to_str(rule.instruction.dir),
            state_to_str(rule.instruction.next_state),
            steps,
            diffs,
            diffs2
        );
    }
}

fn check_min_changed_sequences(
    min_changed_sequences: &[i64],
    execution_records: &[RawExecutionRecord],
    debug: bool,
) -> Option<()> {
    // 各minポジション変化点の間の実行履歴ブロックを取得
    let rules_all = execution_records
        .iter()
        .map(|r| r.rule)
        .collect::<Vec<Rule>>();
    let rules_list = min_changed_sequences
        .windows(2)
        .map(|w| &rules_all[w[0] as usize..w[1] as usize])
        .collect::<Vec<_>>();
    let datas_list = min_changed_sequences
        .iter()
        .map(|i| execution_records[*i as usize - 1].data.as_slice())
        .collect::<Vec<_>>();

    debug_println!(
        debug,
        "rule block lengths per cycle: {:?}",
        rules_list.iter().map(|r| r.len()).collect::<Vec<_>>()
    );
    let repeated_execution_range = find_growing_repeat_block(&rules_list);
    if repeated_execution_range.is_none() {
        debug_println!(debug, "=> no growing repeat block in rules: unresolved");
    }
    let repeated_execution_range = repeated_execution_range?;
    debug_println!(debug, "rule repeat ranges: {:?}", repeated_execution_range);

    // リピートを含むトランジションを構築
    let transitions = collect_blocks_with_repeated_range(
        rules_list[1],
        &repeated_execution_range,
        |s, is_repeated| {
            if is_repeated {
                Transition::RepeatedRulesLinear(RepeatedRulesLinearTransition::new(
                    s,
                    FixedLinearExpr {
                        coefficient: 1,
                        constant: 0,
                    },
                ))
            } else {
                Transition::Rules(RulesTransition::new(s))
            }
        },
    );

    if debug {
        print_execution_cycles(min_changed_sequences, execution_records, &transitions);
    }

    // 各minポジション変化点の間の繰り返しテープブロックを探索
    debug_println!(
        debug,
        "tape lengths per cycle: {:?}",
        datas_list.iter().map(|d| d.len()).collect::<Vec<_>>()
    );
    let repeated_tape_range = find_growing_repeat_block(&datas_list);
    if repeated_tape_range.is_none() {
        debug_println!(debug, "=> no growing repeat block in tape: unresolved");
    }
    let repeated_tape_range = repeated_tape_range?;
    debug_println!(debug, "tape repeat ranges: {:?}", repeated_tape_range);

    let tape_blocks = collect_blocks_with_repeated_range(
        datas_list[1],
        &repeated_tape_range,
        |s, is_repeated| {
            if is_repeated {
                Block::RepeatedSymbolsLinear(RepeatedSymbolsLinearBlock::new(
                    &SymbolsBlock::new(s),
                    FixedLinearExpr {
                        coefficient: 1,
                        constant: 0,
                    },
                ))
            } else {
                Block::Symbols(SymbolsBlock::new(s))
            }
        },
    );

    let mut context = Context {
        step: 0,
        position: CounterExpr::Constant(0),
    };

    let mut tape = Tape::new(tape_blocks);
    let mut tape_next = tape.clone();
    for tape_block in tape_next.blocks_mut() {
        if let Block::RepeatedSymbolsLinear(repeated_block) = tape_block {
            repeated_block.set_repeat_count(repeated_block.repeat_count() + 1);
        }
    }
    debug_println!(debug, "\n{}", "=".repeat(80));
    debug_println!(
        debug,
        "TAPE BLOCKS   [symbols](repeat count), | = block boundary"
    );
    debug_println!(debug, "position: {}", context.position);
    debug_println!(
        debug,
        "initial   {}",
        tape.debug_with_position(context.position)
    );
    debug_println!(debug, "expected {}", tape_next);
    for (i, transition) in transitions.iter().enumerate() {
        debug_println!(debug, "  --- transition {} ---", i + 1);
        debug_println!(
            debug,
            "  before  {}",
            tape.debug_with_position(context.position)
        );
        debug_println!(debug, "  input   {}", transition.input_tape());
        debug_println!(debug, "  output  {}", transition.output_tape());
        let io_range = transition.io_range() + context.position;
        debug_println!(
            debug,
            "  io_range {}..={} (tape coordinates, inclusive)",
            io_range.start,
            io_range.end
        );
        let result = process_with_debug(&mut context, &mut tape, transition, debug);
        if result.is_none() {
            debug_println!(
                debug,
                "  after   {}",
                tape.debug_with_position(context.position)
            );
            debug_println!(
                debug,
                "final after (input mismatch)\n          {}",
                tape.debug_with_position(context.position)
            );
            return None;
        }
        debug_println!(
            debug,
            "  after   {}",
            tape.debug_with_position(context.position)
        );
    }
    debug_println!(
        debug,
        "final after\n          {}",
        tape.debug_with_position(context.position)
    );
    debug_println!(debug, "expected {}", tape_next);

    // テープの繰り返しブロックを1回増やした状態で、同じか比較する
    let is_equal = Tape::compare_with_debug(&tape, &tape_next, debug);
    debug_println!(
        debug,
        "final compare: {}",
        if is_equal { "MATCH" } else { "MISMATCH" }
    );
    if is_equal { Some(()) } else { None }
}

fn format_record_tape(record: &RawExecutionRecord, display_min: i64, display_max: i64) -> String {
    let mut tape = format!(
        "{:>4}<>{:<4} ... ",
        record.min_position, record.max_position
    );
    for position in display_min..=display_max {
        let c = if position < record.min_position || position > record.max_position {
            " "
        } else {
            let symbol = record.data[(position - record.min_position) as usize];
            symbol_to_str(symbol)
        };
        // Keep every absolute tape coordinate in a fixed-width column.
        if position == 0 {
            tape.push('|');
        }
        if position == record.position {
            tape.push_str(&format!("*{c}"));
        } else {
            tape.push_str(&format!(" {c}"));
        }
    }
    tape
}

fn print_execution_cycles(
    boundaries: &[i64],
    records: &[RawExecutionRecord],
    transitions: &[Transition],
) {
    use yan_busy_beaver::base::state_to_str;

    let Some(last_window) = boundaries.windows(2).take(3).next_back() else {
        return;
    };
    let last_record = &records[last_window[1] as usize - 1];
    let display_min = last_record.min_position;
    let display_max = last_record.max_position;

    println!("step is 1-based; *symbol is the head; blank is outside min/max");
    println!("tape display range: {display_min}..={display_max}");
    for (cycle, window) in boundaries.windows(2).take(3).enumerate() {
        let mut cursor = window[0] as usize;
        println!(
            "\n  CYCLE {}   n={}   steps {}..={}",
            cycle + 1,
            cycle,
            cursor + 1,
            window[1]
        );
        println!("{}", "=".repeat(80));
        for transition in transitions {
            let (rule_count, repeats, repeated) = match transition {
                Transition::Rules(t) => (t.rules.len(), 1, false),
                Transition::RepeatedRulesLinear(t) => (
                    t.rule_block.rules.len(),
                    (t.repeat_count.coefficient * cycle as i64 + t.repeat_count.constant) as usize,
                    true,
                ),
            };
            if repeated {
                println!("    <<<<<<<<<<<");
            }
            for repetition in 0..repeats {
                for _ in 0..rule_count {
                    let record = &records[cursor];
                    println!(
                        "s:{:3} {}{} {}{}{}{:2} {}",
                        cursor + 1,
                        state_to_str(record.rule.current_state),
                        symbol_to_str(record.rule.read_symbol),
                        symbol_to_str(record.rule.instruction.write_symbol),
                        dir_to_str(record.rule.instruction.dir),
                        state_to_str(record.rule.instruction.next_state),
                        if repeated {
                            repetition.to_string()
                        } else {
                            "  ".to_string()
                        },
                        format_record_tape(record, display_min, display_max),
                    );
                    cursor += 1;
                }
            }
            if repeated {
                println!("    >>>>>>>>>>>");
            }
        }
        debug_assert_eq!(cursor, window[1] as usize);
    }
}

fn print_csv_progress(completed: usize, total: usize, counts: [usize; 2]) -> std::io::Result<()> {
    use std::io::Write;
    let filled = if total == 0 {
        30
    } else {
        ((completed as u128 * 30) / total as u128) as usize
    };
    let mut stderr = std::io::stderr().lock();
    write!(
        stderr,
        "\r[{}{}] {}/{} (d:{} u:{})",
        "=".repeat(filled),
        " ".repeat(30 - filled),
        completed,
        total,
        counts[0],
        counts[1]
    )?;
    stderr.flush()
}

fn check_csv(
    input: &std::path::Path,
    output: &std::path::Path,
    max_steps: usize,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(input)?;
    let column = reader
        .headers()?
        .iter()
        .position(|h| h == "pattern")
        .ok_or("Missing pattern column")?;
    let records_start = reader.position().clone();
    let total = reader
        .records()
        .try_fold(0usize, |count, row| row.map(|_| count + 1))?;
    reader.seek(records_start)?;
    std::fs::create_dir_all(output)?;
    let input_path = input.canonicalize()?;
    for name in ["decided.csv", "unresolved.csv"] {
        let destination = output.join(name);
        if destination.exists() && destination.canonicalize()? == input_path {
            return Err("Output CSV would overwrite the input".into());
        }
    }
    eprintln!("CSV input:      {}", input.display());
    eprintln!("CSV decided:    {}", output.join("decided.csv").display());
    eprintln!(
        "CSV unresolved: {}",
        output.join("unresolved.csv").display()
    );
    let mut decided = csv::Writer::from_path(output.join("decided.csv"))?;
    let mut unresolved = csv::Writer::from_path(output.join("unresolved.csv"))?;
    for writer in [&mut decided, &mut unresolved] {
        writer.write_record(["pattern", "result", "max_steps"])?;
    }
    let mut counts = [0usize; 2];
    print_csv_progress(0, total, counts)?;
    for (index, row) in reader.records().enumerate() {
        let row = row?;
        let pattern = row
            .get(column)
            .filter(|s| !s.trim().is_empty())
            .ok_or("Empty pattern")?;
        // Preserve unexpected failures instead of recording them as unresolved.
        let result = std::panic::catch_unwind(|| {
            check_with_reverse(pattern, max_steps, debug, ReverseMode::Auto)
        });
        let matched = match result {
            Ok(value) => value,
            Err(payload) => {
                decided.flush()?;
                unresolved.flush()?;
                eprintln!("CSV record {} failed: {}", index + 1, pattern);
                std::panic::resume_unwind(payload);
            }
        };
        let limit = max_steps.to_string();
        if matched {
            decided.write_record([pattern, "loop_candidate_detected", &limit])?;
            counts[0] += 1;
        } else {
            unresolved.write_record([pattern, "unresolved", &limit])?;
            counts[1] += 1;
        }
        print_csv_progress(index + 1, total, counts)?;
    }
    eprintln!();
    decided.flush()?;
    unresolved.flush()?;
    eprintln!(
        "CSV summary:    decided={} unresolved={} (total {})",
        counts[0], counts[1], total
    );
    println!(
        "{}: decided={} unresolved={} -> {}",
        input.display(),
        counts[0],
        counts[1],
        output.display()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if let Some(input) = args.csv {
        let output = args.output_dir.unwrap_or_else(|| {
            input
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("03_linear_step_growth_loop")
        });
        return check_csv(
            &input,
            &output,
            args.max_steps.unwrap_or(1000),
            args.debug.unwrap_or(false),
        );
    }
    let pattern_or_state_num = args.pattern_or_state_num.expect("required by clap");
    match pattern_or_state_num.parse::<u8>() {
        Ok(state) => {
            let base = std::path::PathBuf::from(format!("results/bb_quest_{state}"));
            let input = base.join("02_check_loop/unresolved.csv");
            check_csv(
                &input,
                &base.join("03_linear_step_growth_loop"),
                args.max_steps.unwrap_or(1000),
                args.debug.unwrap_or(false),
            )?;
        }
        Err(_) => {
            let pattern = pattern_or_state_num;
            let max_steps = args.max_steps.unwrap_or(100);
            let debug = args.debug.unwrap_or(true);
            let matched = check_with_reverse(&pattern, max_steps, debug, args.reverse);
            println!(
                "{}: {}",
                pattern,
                if matched {
                    "loop_candidate_detected"
                } else {
                    "unresolved"
                }
            );
        }
    }
    Ok(())
}

// 指定されたパターンに対して実行履歴を取得する関数
fn get_execution_records(pattern: &str, max_steps: usize) -> Vec<RawExecutionRecord> {
    let mut tape = vec![Symbol::Zero; 2 * max_steps + 1];

    let rule_table = RuleTable::from_pattern(pattern.to_string());

    let mut current_state: State = 0;

    let offset = max_steps as i64;

    let mut history = vec![];
    let mut current_position = 0_i64;
    let mut max_position = 0_i64;
    let mut min_position = 0_i64;

    let mut current_symbol = tape[offset as usize];

    for step in 0..max_steps as i64 {
        let rule = rule_table.get_rule(current_state, current_symbol);

        let inst = rule.instruction;
        tape[(current_position + offset) as usize] = inst.write_symbol;
        current_state = inst.next_state;
        current_position += inst.dir.delta();
        current_symbol = tape[(current_position + offset) as usize];

        if current_position > max_position {
            max_position = current_position;
        }
        if current_position < min_position {
            min_position = current_position;
        }
        history.push(RawExecutionRecord {
            step,
            rule: *rule,
            max_position,
            min_position,
            position: current_position,
            data: tape[(min_position + offset) as usize..=(max_position + offset) as usize]
                .to_vec(),
        });

        assert!(current_state != HALT_STATE);
    }
    history
}

// 指定された実行履歴に対して、minポジションが変化したステップの等差数列を見つける関数
fn find_min_changed_sequences_by_rule(
    execution_records: &[RawExecutionRecord],
    max_steps: usize,
    debug: bool,
) -> Option<Vec<Vec<i64>>> {
    // min_positionが変わったところを、Ruleグループごとに集約
    let min_changed_by_rule = execution_records
        .windows(2)
        .filter(|w| w[0].min_position != w[1].min_position)
        .map(|w| w[1].clone())
        .map(|r| (r.rule, r.step))
        .into_group_map();

    // 各Ruleグループごとに、min_positionが変わったステップの差分が等差数列になっているグループを見つける
    let mut min_changed_sequences = vec![];
    for min_changed in min_changed_by_rule.values() {
        let sequences = find_sequence_with_arithmetic_differences(
            &min_changed
                .iter()
                .copied()
                .collect::<std::collections::HashSet<i64>>(),
            2,
        );
        min_changed_sequences.extend(sequences);
    }

    // 等差数列の差分が正で、次の変化が最大ステップを超えるものだけを残す
    if debug {
        println!("MIN-CHANGED SEQUENCE CANDIDATES (steps 1-based)");
        if min_changed_sequences.is_empty() {
            println!(
                "  (no sequence with 2nd-order arithmetic differences; needs at least 5 min-changed steps per rule)"
            );
        }
    }
    min_changed_sequences.retain(|seq| {
        let diff2 = get_diff_for_sequence_with_arithmetic_differences(seq, 2);
        let next = next_for_sequence_with_arithmetic_differences(seq, 2);
        let keep = diff2.is_some_and(|d| d > 0) && next > max_steps as i64;
        if debug {
            let diffs1 = seq.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
            let diffs2 = diffs1.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
            let verdict = if keep {
                "kept".to_string()
            } else if !diff2.is_some_and(|d| d > 0) {
                format!("rejected: 2nd-order diff {:?} is not positive", diff2)
            } else {
                format!(
                    "rejected: next change {} <= max_steps {}",
                    next + 1,
                    max_steps
                )
            };
            println!(
                "  steps {:?}
    diffs {:?}
    diffs2 {:?}  next {}  => {}",
                seq.iter().map(|s| s + 1).collect::<Vec<_>>(),
                diffs1,
                diffs2,
                next + 1,
                verdict
            );
        }
        keep
    });

    if min_changed_sequences.is_empty() {
        return None;
    }
    Some(min_changed_sequences)
}

// 3つのシーケンスに対して、成長する繰り返しブロックを見つける関数
fn find_growing_repeat_block<T>(sequences: &[&[T]]) -> Option<Vec<Range<usize>>>
where
    T: PartialEq + Copy + Debug,
{
    if sequences.len() < 3 {
        return None;
    }
    let previous_seq = sequences[0];
    let current_seq = sequences[1];
    let next_seq = sequences[2];

    if next_seq.len() <= current_seq.len() || current_seq.len() <= previous_seq.len() {
        return None;
    }

    if next_seq.len() - current_seq.len() != current_seq.len() - previous_seq.len() {
        return None;
    }

    if previous_seq.is_empty() {
        return None;
    }

    if previous_seq.is_empty() || current_seq.is_empty() || next_seq.is_empty() {
        return None;
    }

    let mut repeat_block_ranges = vec![];
    let mut previous_seq = previous_seq;
    let mut current_seq = current_seq;
    let mut next_seq = next_seq;
    let mut offset = 0;
    loop {
        let first_diff_position = previous_seq
            .iter()
            .zip(current_seq)
            .position(|(a, b)| a != b)
            .unwrap_or(previous_seq.len());
        let second_diff_position = current_seq
            .iter()
            .zip(next_seq)
            .position(|(a, b)| a != b)
            .unwrap_or(current_seq.len());

        if first_diff_position == 0 && second_diff_position == 0 {
            return None;
        }
        if second_diff_position < first_diff_position {
            return None;
        }

        let block_length = second_diff_position - first_diff_position;

        if block_length > 0 {
            if next_seq.len() < second_diff_position + block_length
                || next_seq[second_diff_position..second_diff_position + block_length]
                    != current_seq[first_diff_position..first_diff_position + block_length]
            {
                return None;
            }
            repeat_block_ranges.push(offset + first_diff_position..offset + second_diff_position);
        }

        offset += second_diff_position;
        previous_seq = &previous_seq[first_diff_position..];
        current_seq = &current_seq[second_diff_position..];
        next_seq = &next_seq[second_diff_position + block_length..];
        if previous_seq.is_empty() {
            break;
        }
    }

    if repeat_block_ranges.is_empty() {
        return None;
    }

    for (i, seq) in sequences.iter().enumerate() {
        let created = collect_blocks_with_repeated_range(
            sequences[1],
            &repeat_block_ranges,
            |seq, is_repeated| seq.to_vec().repeat(if is_repeated { i } else { 1 }),
        )
        .into_iter()
        .flatten()
        .collect::<Vec<T>>();

        if *seq != created {
            return None;
        }
    }
    Some(repeat_block_ranges)
}

fn collect_blocks_with_repeated_range<T, S, F>(
    sequences: &[T],
    repeated_ranges: &Vec<Range<usize>>,
    from_t: F,
) -> Vec<S>
where
    F: Fn(&[T], bool) -> S,
{
    let mut work = 0;
    let mut ret = vec![];
    for repeated_range in repeated_ranges {
        // 静的トランジションをプッシュ
        if repeated_range.start > work {
            ret.push(from_t(&sequences[work..repeated_range.start], false));
        }
        // 繰り返しブロックをプッシュ
        ret.push(from_t(
            &sequences[repeated_range.start..repeated_range.end],
            true,
        ));
        work = repeated_range.end;
    }
    // 残りの静的トランジションをプッシュ
    if work < sequences.len() {
        ret.push(from_t(&sequences[work..], false));
    }
    ret
}
