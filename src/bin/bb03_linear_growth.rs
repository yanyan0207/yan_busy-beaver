use std::ops::Range;

use clap::Parser;
use itertools::Itertools;
use yan_busy_beaver::algorithm::arithmetic_difference::find_sequence_with_arithmetic_differences;
use yan_busy_beaver::algorithm::arithmetic_difference::get_diff_for_sequence_with_arithmetic_differences;
use yan_busy_beaver::algorithm::arithmetic_difference::next_for_sequence_with_arithmetic_differences;
use yan_busy_beaver::base::HALT_STATE;
use yan_busy_beaver::base::Rule;
use yan_busy_beaver::base::RuleTable;
use yan_busy_beaver::base::State;
use yan_busy_beaver::base::Symbol;
use yan_busy_beaver::counter::FixedLinearExpr;
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
    pattern: String,
    /// Maximum steps (default: 100 for a pattern, 1000 for CSV search).
    #[clap(long, default_value_t = 100)]
    max_steps: usize,
}

fn check(pattern: &str, max_steps: usize) -> Option<()> {
    // 実行履歴の取得
    let execution_records = get_execution_records(pattern, max_steps);

    // minポジション変化点の取得(各Ruleグループごと)
    let min_changed_sequences = find_min_changed_sequences_by_rule(&execution_records, max_steps)?;

    // 各minポジション変化点のシーケンスに対して繰り返しブロックを探索
    for min_changed_seq in min_changed_sequences {
        check_min_changed_sequences(&min_changed_seq, &execution_records);
    }
    Some(())
}

fn check_min_changed_sequences(
    min_changed_sequences: &Vec<i64>,
    execution_records: &[RawExecutionRecord],
) -> Option<()> {
    // 各minポジション変化点の間の実行履歴ブロックを取得
    let execution_records_list = min_changed_sequences
        .windows(2)
        .map(|w| &execution_records[w[0] as usize..w[1] as usize])
        .collect::<Vec<_>>();

    // 最初の3つの実行履歴とデータに対して繰り返しブロックを探索
    let prev_executions = execution_records_list[0];
    let current_executions = execution_records_list[1];
    let next_executions = execution_records_list[2];
    let repeated_execution_range = find_growing_repeat_block(
        prev_executions,
        current_executions,
        next_executions,
        |a, b| a.rule == b.rule,
    )?;

    // Rule配列
    let current_rule_seq = current_executions
        .iter()
        .map(|r| r.rule)
        .collect::<Vec<_>>();

    let transitions = collect_blocks_with_repeated_range(
        &current_rule_seq,
        &repeated_execution_range,
        |s, is_repeated| {
            if is_repeated {
                Transition::RepeatedRulesLinear(RepeatedRulesLinearTransition::new(
                    s,
                    FixedLinearExpr::new(1, 0, 0),
                ))
            } else {
                Transition::Rules(RulesTransition::new(s))
            }
        },
    );

    // 各minポジション変化点の間の繰り返しテープブロックを探索
    let repeated_tape_range = find_growing_repeat_block(
        &prev_executions[0].data,
        &current_executions[0].data,
        &next_executions[0].data,
        |a, b| a == b,
    )?;

    print!("{:?} {:?}", min_changed_sequences, repeated_execution_range);
    Some(())
}

fn main() {
    let args = Args::parse();

    println!("pattern {} max_steps {}", args.pattern, args.max_steps);

    // パターンに対してminポジション変化点の繰り返しブロックをチェック
    check(&args.pattern, args.max_steps);
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
    min_changed_sequences = min_changed_sequences
        .into_iter()
        .filter(|seq| {
            get_diff_for_sequence_with_arithmetic_differences(seq, 2).is_some_and(|diff| diff > 0)
        })
        .filter(|seq| next_for_sequence_with_arithmetic_differences(seq, 2) > max_steps as i64)
        .collect();

    if min_changed_sequences.is_empty() {
        return None;
    }
    Some(min_changed_sequences)
}

// 3つのシーケンスに対して、成長する繰り返しブロックを見つける関数
fn find_growing_repeat_block<T, F>(
    previous_seq: &[T],
    current_seq: &[T],
    next_seq: &[T],
    eq: F,
) -> Option<Vec<Range<usize>>>
where
    F: Fn(&T, &T) -> bool,
{
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
            .position(|(a, b)| !eq(a, b))
            .unwrap_or(previous_seq.len());
        let second_diff_position = current_seq
            .iter()
            .zip(next_seq)
            .position(|(a, b)| !eq(a, b))
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
                    .iter()
                    .zip(&current_seq[first_diff_position..first_diff_position + block_length])
                    .any(|(a, b)| !eq(a, b))
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
    Some(repeat_block_ranges)
}

fn collect_blocks_with_repeated_range<T, S, F>(
    sequences: &[T],
    repeated_range: &Vec<Range<usize>>,
    from_t: F,
) -> Vec<S>
where
    F: Fn(&[T], bool) -> S,
{
    let mut work = 0;
    let mut ret = vec![];
    for i in repeated_range {
        // 静的トランジションをプッシュ
        if i.start > work {
            ret.push(from_t(&sequences[work..i.start], false));
        }
        // 繰り返しブロックをプッシュ
        ret.push(from_t(&sequences[i.start..i.end], true));
        work = i.end;
    }
    // 残りの静的トランジションをプッシュ
    if work < sequences.len() {
        ret.push(from_t(&sequences[work..], false));
    }
    ret
}
