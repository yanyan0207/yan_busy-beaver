//! Detect a block that repeats one more time in each of three consecutive sequences.

use std::fmt::Debug;
use std::ops::Range;

/// Given three consecutive sequences whose lengths grow by a constant amount, find the
/// ranges in the middle sequence that repeat `i` times in sequence `i` (0-based).
pub fn find_growing_repeat_block<T>(sequences: &[&[T]]) -> Option<Vec<Range<usize>>>
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

/// Split `sequences` into blocks, marking the ones inside `repeated_ranges` as repeated.
pub fn collect_blocks_with_repeated_range<T, S, F>(
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
