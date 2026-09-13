//! Plain Turing machine simulation that records the tape after every step.

use crate::base::{
    HALT_STATE, Rule, RuleTable, State, Symbol, dir_to_str, state_to_str, symbol_to_str,
};

/// One executed step: the rule applied, the head position after the move, and the
/// tape contents between the minimum and maximum positions visited so far.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawExecutionRecord {
    pub step: i64,
    pub rule: Rule,
    pub position: i64,
    pub max_position: i64,
    pub min_position: i64,
    pub data: Vec<Symbol>,
}

/// Mirror a pattern by swapping every L and R.
pub fn reverse_pattern(pattern: &str) -> String {
    pattern
        .chars()
        .map(|c| match c {
            'L' => 'R',
            'R' => 'L',
            c => c,
        })
        .collect()
}

/// Run `pattern` for `max_steps` steps from a blank tape and return one record per step.
/// Panics if the machine halts within `max_steps`.
pub fn get_execution_records(pattern: &str, max_steps: usize) -> Vec<RawExecutionRecord> {
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

/// Format a rule as `<state><read> <write><dir><next>`, e.g. `A0 1RB`.
pub fn format_rule(rule: &Rule) -> String {
    format!(
        "{}{} {}{}{}",
        state_to_str(rule.current_state),
        symbol_to_str(rule.read_symbol),
        symbol_to_str(rule.instruction.write_symbol),
        dir_to_str(rule.instruction.dir),
        state_to_str(rule.instruction.next_state)
    )
}

/// Render the tape of one record on a fixed absolute coordinate grid: `*` marks the
/// head, `|` marks position 0, and cells outside min/max are blank.
pub fn format_record_tape(
    record: &RawExecutionRecord,
    display_min: i64,
    display_max: i64,
) -> String {
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

/// Print every executed step in the same layout as `print_execution_cycles`,
/// marking the steps where min_position moved.
pub fn print_execution_records(records: &[RawExecutionRecord]) {
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
            "s:{:3} {}    {}",
            index + 1,
            format_rule(&record.rule),
            format_record_tape(record, display_min, display_max),
        );
    }
}
