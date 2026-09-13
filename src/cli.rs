//! Shared command-line plumbing for the bb_* stages: progress display and the
//! "read unresolved.csv, split into decided.csv / unresolved.csv" pipeline.

use std::io::Write;
use std::panic::AssertUnwindSafe;
use std::path::Path;

/// Redraw a single-line stderr progress bar; `counts` is `[decided, unresolved]`.
pub fn print_csv_progress(
    completed: usize,
    total: usize,
    counts: [usize; 2],
) -> std::io::Result<()> {
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

/// Read every `pattern` from the CSV at `input`, call `check` on each, and write the
/// matches to `output/decided.csv` (with `decided_result` as the result column) and the
/// rest to `output/unresolved.csv`. A panic inside `check` flushes both files, reports
/// the offending record, and is then re-raised.
pub fn run_csv_stage<F>(
    input: &Path,
    output: &Path,
    max_steps: usize,
    decided_result: &str,
    mut check: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut(&str) -> bool,
{
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
        let result = std::panic::catch_unwind(AssertUnwindSafe(|| check(pattern)));
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
            decided.write_record([pattern, decided_result, &limit])?;
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
