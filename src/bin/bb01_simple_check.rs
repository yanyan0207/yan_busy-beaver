use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};
use yan_busy_beaver::base::{
    Direction, HALT_STATE, Instruction, Rule, RuleTable, State, Symbol, state_to_str,
};

#[derive(Parser)]
struct Args {
    pattern_or_state_num: String,
    /// Maximum steps (default: 100 for a pattern, 1000 for CSV search).
    #[clap(long)]
    max_steps: Option<usize>,
}

struct Result {
    steps: i64,
    last_rule: Rule,
}

fn halt_reachable(table: &RuleTable, start: State) -> bool {
    let mut pending = vec![start];
    let mut visited = vec![false; table.rules.len() / 2];
    while let Some(state) = pending.pop() {
        if state == HALT_STATE {
            return true;
        }
        if visited[state as usize] {
            continue;
        }
        visited[state as usize] = true;
        for symbol in [Symbol::Zero, Symbol::One] {
            pending.push(table.get_rule(state, symbol).instruction.next_state);
        }
    }
    false
}

fn equivalent_states(table: &RuleTable, states: &[State]) -> bool {
    [Symbol::Zero, Symbol::One].into_iter().all(|symbol| {
        let first = table.get_rule(states[0], symbol).instruction;
        let instructions: Vec<_> = states
            .iter()
            .map(|&state| table.get_rule(state, symbol).instruction)
            .collect();
        instructions.iter().all(|inst| {
            inst.next_state != HALT_STATE
                && inst.write_symbol == first.write_symbol
                && inst.dir == first.dir
        }) && (instructions
            .iter()
            .all(|inst| inst.next_state == first.next_state)
            || instructions
                .iter()
                .all(|inst| states.contains(&inst.next_state)))
    })
}

fn skippable(table: &RuleTable, stack: &[Rule]) -> bool {
    // Keep the right-moving representative of each reflected machine.
    if table.get_rule(0, Symbol::Zero).instruction.dir == Direction::Left {
        return true;
    }

    // The newest rule is reached before any remaining undefined rule.
    // If its target cannot reach HALT, extending this branch cannot help.
    if !halt_reachable(table, stack.last().unwrap().instruction.next_state) {
        return true;
    }

    let state_num = table.rules.len() / 2;
    let mut referenced = vec![false; state_num];
    for rule in &table.rules {
        if rule.instruction.next_state != HALT_STATE {
            referenced[rule.instruction.next_state as usize] = true;
        }
    }
    // Keep state names contiguous from B, as in the reference search.
    if referenced[1..].windows(2).any(|pair| !pair[0] && pair[1]) {
        return true;
    }

    // Omit machines reducible by merging two or three noninitial states.
    for i in 1..state_num {
        for j in i + 1..state_num {
            if equivalent_states(table, &[i as State, j as State]) {
                return true;
            }
            for k in j + 1..state_num {
                if equivalent_states(table, &[i as State, j as State, k as State]) {
                    return true;
                }
            }
        }
    }
    false
}

fn print_tape(
    tape: &[Symbol],
    current_position: i64,
    offset: usize,
    min_position: i64,
    max_position: i64,
) {
    print!("min:{} ... ", min_position);
    for pos in min_position..=max_position {
        let symbol = tape[(pos + offset as i64) as usize];
        let c = match symbol {
            Symbol::Zero => "0",
            Symbol::One => "1",
        };

        if pos == current_position {
            print!("[{}]", c);
        } else {
            print!("{}", c);
        }
    }
    print!("... max:{}", max_position);
}

fn check_pattern(pattern: &str, max_steps: usize, debug: bool) -> Option<Result> {
    let mut tape = vec![Symbol::Zero; 2 * max_steps + 1];

    let rule_table = RuleTable::from_pattern(pattern.to_string());

    let mut current_state: State = 0;

    let offset = max_steps;
    let mut current_position = 0_i64;
    let mut max_position = 0_i64;
    let mut min_position = 0_i64;

    let mut current_symbol = tape[offset];

    for step in 0..max_steps {
        let rule = rule_table.get_rule(current_state, current_symbol);
        if debug {
            print!(
                "step: {} state:{} symbol:{:?} pos:{} ",
                step + 1,
                state_to_str(current_state),
                current_symbol,
                current_position
            );
            print_tape(&tape, current_position, offset, min_position, max_position);
        }

        let inst = rule.instruction;
        tape[(current_position + offset as i64) as usize] = inst.write_symbol;
        current_state = inst.next_state;
        current_position += inst.dir.delta();
        current_symbol = tape[(current_position + offset as i64) as usize];

        if current_position > max_position {
            max_position = current_position;
        }
        if current_position < min_position {
            min_position = current_position;
        }

        if debug {
            print!(" => ");
            print_tape(&tape, current_position, offset, min_position, max_position);
            println!();
        }

        if current_state == HALT_STATE {
            return Some(Result {
                steps: (step + 1) as i64,
                last_rule: *rule,
            });
        }
    }
    None
}

fn make_pattern_candidates(state_num: State) -> Vec<Instruction> {
    let mut patterns = vec![];
    for write_symbol in [Symbol::Zero, Symbol::One] {
        for dir in [Direction::Left, Direction::Right] {
            for next_state in 0..state_num as State {
                let inst = Instruction {
                    write_symbol,
                    dir,
                    next_state,
                };
                patterns.push(inst);
            }
        }
    }
    patterns
}

fn stack_to_pattern(stack: &[Rule], num_state: usize) -> String {
    const HALT_PATTERN: &str = "0RH";
    let mut pattern_list = vec![HALT_PATTERN.to_string(); num_state * 2];
    for rule in stack {
        let state_index = rule.current_state as usize;
        let symbol_index = match rule.read_symbol {
            Symbol::Zero => 0,
            Symbol::One => 1,
        };
        pattern_list[state_index * 2 + symbol_index] = rule.instruction.to_str();
    }
    pattern_list.join(" ")
}

fn pop(stack: &mut Vec<Rule>) -> Option<()> {
    stack.pop();
    if stack.is_empty() {
        return None;
    }
    Some(())
}

fn push(stack: &mut Vec<Rule>, pattern_candidates: &[Instruction], state: State, symbol: Symbol) {
    stack.push(Rule {
        current_state: state,
        read_symbol: symbol,
        instruction: pattern_candidates[0],
    });
}

fn next(stack: &mut Vec<Rule>, pattern_candidates: &[Instruction]) -> Option<()> {
    loop {
        let last_index = pattern_candidates
            .iter()
            .position(|x| x == &stack.last().unwrap().instruction)
            .unwrap();
        if last_index == pattern_candidates.len() - 1 {
            pop(stack)?;
        } else {
            let next_inst = pattern_candidates[last_index + 1];
            stack.last_mut().unwrap().instruction = next_inst;
            return Some(());
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let Some(state_num) = args.pattern_or_state_num.parse::<usize>().ok() else {
        let max_steps = args.max_steps.unwrap_or(100);
        println!("{} {}", args.pattern_or_state_num, max_steps);
        check_pattern(&args.pattern_or_state_num, max_steps, true);
        return Ok(());
    };
    let max_steps = args.max_steps.unwrap_or(1000);

    let result_dir = format!("results/bb_quest_{state_num}/01_simple_search");
    fs::create_dir_all(&result_dir)?;
    let mut decided_writer = BufWriter::new(File::create(format!("{result_dir}/decided.csv"))?);
    let mut unresolved_writer =
        BufWriter::new(File::create(format!("{result_dir}/unresolved.csv"))?);
    let mut skipped_writer = BufWriter::new(File::create(format!("{result_dir}/skipped.csv"))?);
    writeln!(decided_writer, "pattern,result,steps")?;
    writeln!(unresolved_writer, "pattern,result,steps")?;
    writeln!(skipped_writer, "pattern,result")?;

    let mut stack = vec![];
    let pattern_candidates = make_pattern_candidates(state_num as State);
    push(&mut stack, &pattern_candidates, 0, Symbol::Zero);

    loop {
        let pattern = stack_to_pattern(&stack, state_num);
        let table = RuleTable::from_pattern(pattern.clone());
        if skippable(&table, &stack) {
            writeln!(skipped_writer, "{pattern},skipped")?;
            if next(&mut stack, &pattern_candidates).is_none() {
                break;
            }
            continue;
        }
        // Generated patterns contain only instruction characters and spaces,
        // so these CSV fields do not require quoting or escaping.
        if let Some(result) = check_pattern(&pattern, max_steps, false) {
            writeln!(decided_writer, "{pattern},halt,{}", result.steps)?;
            if stack.len() == state_num * 2 - 1 {
                if next(&mut stack, &pattern_candidates).is_none() {
                    break;
                };
            } else {
                push(
                    &mut stack,
                    &pattern_candidates,
                    result.last_rule.current_state,
                    result.last_rule.read_symbol,
                );
            }
        } else {
            writeln!(unresolved_writer, "{pattern},unresolved,{max_steps}")?;
            if next(&mut stack, &pattern_candidates).is_none() {
                break;
            }
        }
    }
    decided_writer.flush()?;
    unresolved_writer.flush()?;
    skipped_writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(pattern: &str) -> RuleTable {
        RuleTable::from_pattern(pattern.to_string())
    }

    #[test]
    fn reachability_handles_cycles_and_checks_both_symbols() {
        let closed = table("0RB 0RH 0RB 1LB");
        assert!(!halt_reachable(&closed, 1));
        assert!(skippable(&closed, &closed.rules[..1]));
        let escape = table("0RB 0RH 0RB 1LA");
        assert!(halt_reachable(&escape, 1));
        assert!(!skippable(&escape, &escape.rules[..1]));
    }

    #[test]
    fn state_name_gaps_are_skipped_but_contiguous_names_are_kept() {
        let gap = table("0RC 0RH 0RH 0RH 0RH 0RH");
        assert!(skippable(&gap, &gap.rules[..1]));
        let contiguous = table("0RB 0RH 0RC 0RH 0RH 0RH");
        assert!(!skippable(&contiguous, &contiguous.rules[..1]));
    }

    #[test]
    fn equivalent_pairs_require_matching_actions_and_nonhalting_rules() {
        let equivalent = table("0RB 0RH 1RA 0LA 1RA 0LA");
        assert!(equivalent_states(&equivalent, &[1, 2]));
        assert!(skippable(&equivalent, &equivalent.rules[..1]));
        for pattern in [
            "0RB 0RH 1RA 0LA 0RA 0LA",
            "0RB 0RH 1RA 0LA 1LA 0LA",
            "0RB 0RH 1RA 0LA 1RA 0RH",
        ] {
            assert!(!equivalent_states(&table(pattern), &[1, 2]));
        }
    }

    #[test]
    fn three_state_equivalence_can_hold_without_an_equivalent_pair() {
        let cycle = table("0RB 0RH 1RC 0LA 1RD 0LA 1RB 0LA");
        for pair in [[1, 2], [1, 3], [2, 3]] {
            assert!(!equivalent_states(&cycle, &pair));
        }
        assert!(equivalent_states(&cycle, &[1, 2, 3]));
        assert!(skippable(&cycle, &cycle.rules[..1]));
    }
}
