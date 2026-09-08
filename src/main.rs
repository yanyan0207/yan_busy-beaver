use clap::Parser;
use yan_busy_beaver::base::{HALT_STATE, RuleTable, State, Symbol, state_to_str};

#[derive(Parser)]
struct Args {
    pattern: String,
    #[clap(long, default_value_t = 100)]
    max_steps: usize,
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
        let symbol = tape[(pos as usize) + offset];
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

fn main() {
    let args = Args::parse();

    println!("{} {}", args.pattern, args.max_steps);

    let mut tape = vec![Symbol::Zero; 2 * args.max_steps + 1];

    let rule_table = RuleTable::from_pattern(args.pattern);

    let mut current_state: State = 0;

    let offset = args.max_steps;
    let mut current_position = 0_i64;
    let mut max_position = 0_i64;
    let mut min_position = 0_i64;

    let mut current_symbol = tape[current_position as usize];

    for step in 0..args.max_steps {
        // Simulation logic goes here
        let rule = rule_table.get_rule(current_state, current_symbol);
        print!(
            "step: {} state:{} symbol:{:?} pos:{} ",
            step + 1,
            state_to_str(current_state),
            current_symbol,
            current_position
        );
        print_tape(&tape, current_position, offset, min_position, max_position);

        let inst = rule.instruction;
        tape[current_position as usize + offset] = inst.write_symbol;
        current_state = inst.next_state;
        current_position += inst.dir.delta();
        current_symbol = tape[current_position as usize + offset];

        print!(" => ");
        print_tape(&tape, current_position, offset, min_position, max_position);
        println!();

        if current_position > max_position {
            max_position = current_position;
        }
        if current_position < min_position {
            min_position = current_position;
        }

        if current_state == HALT_STATE {
            break;
        }
    }
}
