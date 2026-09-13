use clap::Parser;
use itertools::Itertools;
use yan_busy_beaver::algorithm::arithmetic_difference::find_sequence_with_arithmetic_differences;
use yan_busy_beaver::algorithm::arithmetic_difference::get_diff_for_sequence_with_arithmetic_differences;
use yan_busy_beaver::algorithm::arithmetic_difference::next_for_sequence_with_arithmetic_differences;
use yan_busy_beaver::algorithm::repeat_block::collect_blocks_with_repeated_range;
use yan_busy_beaver::algorithm::repeat_block::find_growing_repeat_block;
use yan_busy_beaver::base::Rule;
use yan_busy_beaver::block::Block;
use yan_busy_beaver::block::RepeatedSymbolsLinearBlock;
use yan_busy_beaver::block::SymbolsBlock;
use yan_busy_beaver::cli::run_csv_stage;
use yan_busy_beaver::counter::CounterExpr;
use yan_busy_beaver::counter::FixedLinearExpr;
use yan_busy_beaver::debug_println;
use yan_busy_beaver::interpreter::Context;
use yan_busy_beaver::interpreter::process_with_debug;
use yan_busy_beaver::machine::RawExecutionRecord;
use yan_busy_beaver::machine::format_record_tape;
use yan_busy_beaver::machine::format_rule;
use yan_busy_beaver::machine::get_execution_records;
use yan_busy_beaver::machine::print_execution_records;
use yan_busy_beaver::machine::reverse_pattern;
use yan_busy_beaver::tape::Tape;
use yan_busy_beaver::transition::RepeatedRulesLinearTransition;
use yan_busy_beaver::transition::RulesTransition;
use yan_busy_beaver::transition::Transition;
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

/// Print where min_position moved, grouped by rule, and which groups formed an
/// arithmetic sequence that survives until max_steps.
fn print_min_changed_sequences(records: &[RawExecutionRecord]) {
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
            "  {}  steps {:?}  diffs {:?}  diffs2 {:?}",
            format_rule(rule),
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

fn print_execution_cycles(
    boundaries: &[i64],
    records: &[RawExecutionRecord],
    transitions: &[Transition],
) {
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
                        "s:{:3} {}{:2} {}",
                        cursor + 1,
                        format_rule(&record.rule),
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

fn check_csv(
    input: &std::path::Path,
    output: &std::path::Path,
    max_steps: usize,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    run_csv_stage(
        input,
        output,
        max_steps,
        "loop_candidate_detected",
        |pattern| check_with_reverse(pattern, max_steps, debug, ReverseMode::Auto),
    )
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

// 指定された実行履歴に対して、minポジションが変化したステップの等差数列を見つける関数
/// min変化点のステップ列に要求する階差の次数。
/// 2 = 「min変化点の間隔(1階差)が等差数列」= 1サイクルのステップ数が線形に増える。
const DIFFERENCE_ORDER: usize = 2;

/// 階差数列の判定に必要な最小要素数(`find_sequence_with_arithmetic_differences`の仕様)。
const MIN_SEQUENCE_LEN: usize = DIFFERENCE_ORDER + 3;

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
            DIFFERENCE_ORDER,
        );
        min_changed_sequences.extend(sequences);
    }

    // 等差数列の差分が正で、次の変化が最大ステップを超えるものだけを残す
    if debug {
        println!("MIN-CHANGED SEQUENCE CANDIDATES (steps 1-based)");
        if min_changed_sequences.is_empty() {
            println!(
                "  (no sequence with {DIFFERENCE_ORDER}-order arithmetic differences; needs at least {MIN_SEQUENCE_LEN} min-changed steps per rule)"
            );
        }
    }
    min_changed_sequences.retain(|seq| {
        // DIFFERENCE_ORDER階の階差が全て同じ値ならその値、そうでなければNone
        let diff2 = get_diff_for_sequence_with_arithmetic_differences(seq, DIFFERENCE_ORDER);
        // 階差数列を1つ外挿して、次にminが変化するはずのステップ(0-based)を予測
        let next = next_for_sequence_with_arithmetic_differences(seq, DIFFERENCE_ORDER);
        // 階差が正(サイクルが伸び続ける)かつ、次の変化がmax_stepsより先(実行範囲内に反例がない)なら残す
        let keep = diff2.is_some_and(|d| d > 0) && next > max_steps as i64;
        if debug {
            let diffs1 = seq.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
            let diffs2 = diffs1.windows(2).map(|w| w[1] - w[0]).collect::<Vec<_>>();
            let verdict = if keep {
                "kept".to_string()
            } else if !diff2.is_some_and(|d| d > 0) {
                format!("rejected: {DIFFERENCE_ORDER}-order diff {diff2:?} is not positive")
            } else {
                format!(
                    "rejected: next change {} <= max_steps {}",
                    next + 1,
                    max_steps
                )
            };
            println!(
                "  steps {:?}\n    diffs {:?}\n    diffs2 {:?}  next {}  => {}",
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
