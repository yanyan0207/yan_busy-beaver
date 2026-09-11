use std::collections::HashSet;

use itertools::Itertools;

pub fn find_sequence_with_arithmetic_differences(
    arr: &HashSet<i64>,
    difference_order: usize,
) -> Vec<Vec<i64>> {
    let mut arr = arr.iter().copied().collect::<Vec<i64>>();
    arr.sort();

    let mut searched = HashSet::new();
    let mut results = vec![];
    for combination in arr.iter().copied().combinations(difference_order + 3) {
        if searched.contains(&combination) {
            continue;
        }
        // 階差数列になっているか確認
        let diff =
            get_diff_for_sequence_with_arithmetic_differences(&combination, difference_order);
        if diff.is_none() {
            continue;
        }

        let mut result = combination;
        // 次の候補があるか確認
        loop {
            let next = next_for_sequence_with_arithmetic_differences(&result, difference_order);
            if arr.contains(&next) {
                result.push(next);
            } else {
                break;
            }
        }
        if result.is_empty() {
            continue;
        }
        result.windows(difference_order + 3).for_each(|w| {
            searched.insert(w.to_vec());
        });
        results.push(result);
    }
    results
}

pub fn next_for_sequence_with_arithmetic_differences(arr: &[i64], difference_order: usize) -> i64 {
    let arr = arr[arr.len() - (difference_order + 2)..].to_vec();

    let mut nth_last_datas = vec![arr.last().copied().unwrap()];
    for i in 1..=difference_order + 1 {
        nth_last_datas.push(nth_difference(&arr, i).last().copied().unwrap());
    }
    nth_last_datas.sort();

    for i in 0..nth_last_datas.len() - 1 {
        nth_last_datas[i + 1] += nth_last_datas[i];
    }
    nth_last_datas.last().copied().unwrap()
}

pub fn get_diff_for_sequence_with_arithmetic_differences(
    arr: &[i64],
    difference_order: usize,
) -> Option<i64> {
    if arr.len() < difference_order + 3 {
        return None;
    }

    let nth_diffed = nth_difference(arr, difference_order);

    if nth_diffed.iter().all_equal() {
        return Some(nth_diffed[0]);
    }
    None
}

fn nth_difference(arr: &[i64], difference_order: usize) -> Vec<i64> {
    if arr.len() <= difference_order {
        return vec![];
    }

    // n次の階差を取得
    let mut result = arr.to_vec();
    for _ in 0..difference_order {
        result = result.windows(2).map(|w| w[1] - w[0]).collect();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_arithmetic_progression() {
        assert_eq!(
            find_sequence_with_arithmetic_differences(&HashSet::from([1, 2, 3, 4]), 1),
            vec![vec![1, 2, 3, 4]]
        );
    }

    #[test]
    fn computes_differences_for_prediction_window() {
        assert_eq!(nth_difference(&[2, 3, 4], 1), vec![1, 1]);
        assert_eq!(nth_difference(&[2, 3, 4], 2), vec![0]);
    }

    #[test]
    fn returns_empty_when_no_progression_exists() {
        assert!(
            find_sequence_with_arithmetic_differences(&HashSet::from([1, 2, 4, 8]), 1).is_empty()
        );
    }

    #[test]
    fn repeated_values_do_not_loop() {
        let input = HashSet::from([1, 1, 1, 1]);
        assert!(find_sequence_with_arithmetic_differences(&input, 0).is_empty());
        assert!(find_sequence_with_arithmetic_differences(&input, 1).is_empty());
        assert!(find_sequence_with_arithmetic_differences(&HashSet::from([1, 2, 3]), 0).is_empty());
    }

    #[test]
    fn extends_quadratic_sequence() {
        let input = HashSet::from([1, 4, 9, 16, 25, 36]);
        assert!(
            find_sequence_with_arithmetic_differences(&input, 2)
                .contains(&vec![1, 4, 9, 16, 25, 36])
        );
    }
}
