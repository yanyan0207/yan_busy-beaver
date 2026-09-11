#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    Zero,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Left,
    Right,
}

impl Direction {
    pub fn delta(&self) -> i64 {
        match self {
            Direction::Left => -1,
            Direction::Right => 1,
        }
    }
}

pub type State = i8;
pub const HALT_STATE: State = -1;

pub fn state_to_str(state: State) -> &'static str {
    match state {
        0 => "A",
        1 => "B",
        2 => "C",
        3 => "D",
        4 => "E",
        HALT_STATE => "H",
        _ => panic!("Invalid state"),
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub write_symbol: Symbol,
    pub dir: Direction,
    pub next_state: State,
}

impl Instruction {
    pub fn from_str(inst_str: &str, state_num: i64) -> Self {
        assert!(inst_str.len() == 3);
        let inst = Self {
            write_symbol: match inst_str.chars().nth(0).unwrap() {
                '0' => Symbol::Zero,
                '1' => Symbol::One,
                _ => panic!("Invalid symbol"),
            },
            dir: match inst_str.chars().nth(1).unwrap() {
                'L' => Direction::Left,
                'R' => Direction::Right,
                _ => panic!("Invalid direction"),
            },
            next_state: match inst_str.chars().nth(2).unwrap() {
                'A' => 0,
                'B' => 1,
                'C' => 2,
                'D' => 3,
                'E' => 4,
                'H' => HALT_STATE,
                _ => panic!("Invalid state"),
            },
        };
        if inst.next_state as i64 >= state_num {
            panic!("Next state exceeds max state");
        }
        inst
    }

    pub fn to_str(&self) -> String {
        format!(
            "{}{}{}",
            match self.write_symbol {
                Symbol::Zero => '0',
                Symbol::One => '1',
            },
            match self.dir {
                Direction::Left => 'L',
                Direction::Right => 'R',
            },
            state_to_str(self.next_state)
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rule {
    pub current_state: State,
    pub read_symbol: Symbol,
    pub instruction: Instruction,
}

impl Rule {
    pub fn delta(&self) -> i64 {
        self.instruction.dir.delta()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleTable {
    pub rules: Vec<Rule>,
}

impl RuleTable {
    pub fn from_pattern(patterns: String) -> Self {
        let instr_strs: Vec<&str> = patterns.split_whitespace().collect();
        assert!(
            instr_strs.len() >= 2 && instr_strs.len().is_multiple_of(2) && instr_strs.len() < 10
        );
        let state_num = instr_strs.len() as i64 / 2;

        let mut rules = Vec::new();
        for (index, inst_str) in instr_strs.iter().enumerate() {
            let inst = Instruction::from_str(inst_str, state_num);
            rules.push(Rule {
                current_state: index as State / 2, // Replace with actual current state
                read_symbol: match index % 2 {
                    0 => Symbol::Zero,
                    1 => Symbol::One,
                    _ => panic!("Invalid read symbol"),
                }, // Replace with actual read symbol
                instruction: inst,
            });
        }
        Self { rules }
    }

    pub fn get_rule(&self, current_state: State, read_symbol: Symbol) -> &Rule {
        self.rules
            .iter()
            .find(|rule| rule.current_state == current_state && rule.read_symbol == read_symbol)
            .unwrap()
    }
}
