#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use clap::ValueEnum;
use serde::Serialize;
use statrs::distribution::{ContinuousCDF, StudentsT};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Unit {
    Us,
    Ms,
    Ns,
}

impl Unit {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Us => "us",
            Self::Ms => "ms",
            Self::Ns => "ns",
        }
    }

    fn to_us(self, value: f64) -> f64 {
        match self {
            Self::Us => value,
            Self::Ms => value * 1_000.0,
            Self::Ns => value / 1_000.0,
        }
    }

    fn to_seconds(self, value: f64) -> f64 {
        match self {
            Self::Us => value / 1_000_000.0,
            Self::Ms => value / 1_000.0,
            Self::Ns => value / 1_000_000_000.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ProcessOptions {
    pub latency_column: String,
    pub start_column: String,
    pub ok_column: String,
    pub unit: Unit,
    pub warmup_seconds: Option<f64>,
    pub cooldown_seconds: Option<f64>,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            latency_column: "latency_us".to_owned(),
            start_column: "start_us".to_owned(),
            ok_column: "ok".to_owned(),
            unit: Unit::Us,
            warmup_seconds: None,
            cooldown_seconds: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct DropCounts {
    pub warmup: usize,
    pub cooldown: usize,
    pub failed: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct OperationCounts {
    pub total_records: usize,
    /// Successful operations included in the latency statistics.
    pub kept: usize,
    pub dropped: DropCounts,
}

#[derive(Clone, Debug, Serialize)]
pub struct LatencyStats {
    pub min_us: f64,
    pub p50_us: f64,
    pub p90_us: f64,
    pub p99_us: f64,
    pub p99_9_us: f64,
    pub max_us: f64,
    pub mean_us: f64,
    /// Population standard deviation of this run's retained samples.
    pub stddev_us: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunSummary {
    pub file: String,
    pub operations: OperationCounts,
    pub window_seconds: Option<f64>,
    pub throughput_ops_per_second: Option<f64>,
    pub latency: LatencyStats,
    #[serde(skip)]
    latency_values_us: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AcrossRunStats {
    pub n: usize,
    pub unit: String,
    pub mean: f64,
    pub sample_stddev: f64,
    pub ci95_low: f64,
    pub ci95_high: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct JsonReport {
    pub input_unit: String,
    pub latency_unit: &'static str,
    pub window_unit: &'static str,
    pub throughput_unit: &'static str,
    pub runs: Vec<RunSummary>,
    pub across_runs: Option<BTreeMap<String, AcrossRunStats>>,
}

#[derive(Clone, Debug)]
struct RawOperation {
    latency_us: f64,
    start_seconds: Option<f64>,
    ok: bool,
}

#[derive(Clone, Debug)]
struct MetricValue {
    name: &'static str,
    value: Option<f64>,
    unit: &'static str,
}

pub fn process_file(path: &Path, options: &ProcessOptions) -> Result<RunSummary, String> {
    validate_duration("--warmup", options.warmup_seconds)?;
    validate_duration("--cooldown", options.cooldown_seconds)?;

    let file = File::open(path)
        .map_err(|error| format!("{}: cannot open input file: {error}", path.display()))?;
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file);
    let headers = reader
        .headers()
        .map_err(|error| format!("{}: cannot read CSV header: {error}", path.display()))?
        .clone();
    if headers.is_empty() {
        return Err(format!("{}: CSV header row is required", path.display()));
    }

    let latency_index = column_index(&headers, &options.latency_column).ok_or_else(|| {
        format!(
            "{}: missing required column {:?}",
            path.display(),
            options.latency_column
        )
    })?;
    let start_index = column_index(&headers, &options.start_column);
    if start_index.is_none() && options.warmup_seconds.is_some() {
        return Err(format!(
            "{}: --warmup requires start column {:?}, but that column is missing",
            path.display(),
            options.start_column
        ));
    }
    if start_index.is_none() && options.cooldown_seconds.is_some() {
        return Err(format!(
            "{}: --cooldown requires start column {:?}, but that column is missing",
            path.display(),
            options.start_column
        ));
    }
    let ok_index = column_index(&headers, &options.ok_column);

    let mut operations = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|error| {
            let line = csv_error_line(&error).unwrap_or(2);
            format!("{}:{line}: malformed CSV record: {error}", path.display())
        })?;
        let line = record
            .position()
            .map(|position| position.line())
            .unwrap_or(2);
        let latency_text = record.get(latency_index).unwrap_or_default();
        let latency_value = parse_number(latency_text).map_err(|reason| {
            format!(
                "{}:{line}: invalid latency {:?}: {reason}",
                path.display(),
                latency_text
            )
        })?;
        if latency_value < 0.0 {
            return Err(format!(
                "{}:{line}: invalid latency {:?}: latency must be non-negative",
                path.display(),
                latency_text.trim()
            ));
        }
        let latency_us = options.unit.to_us(latency_value);
        if !latency_us.is_finite() {
            return Err(format!(
                "{}:{line}: invalid latency {:?}: converted value is not finite",
                path.display(),
                latency_text.trim()
            ));
        }

        let start_seconds = if let Some(index) = start_index {
            let start_text = record.get(index).unwrap_or_default();
            let start_value = parse_number(start_text).map_err(|reason| {
                format!(
                    "{}:{line}: invalid start time {:?}: {reason}",
                    path.display(),
                    start_text
                )
            })?;
            let seconds = options.unit.to_seconds(start_value);
            if !seconds.is_finite() {
                return Err(format!(
                    "{}:{line}: invalid start time {:?}: converted value is not finite",
                    path.display(),
                    start_text.trim()
                ));
            }
            Some(seconds)
        } else {
            None
        };

        let ok = if let Some(index) = ok_index {
            parse_ok(record.get(index).unwrap_or_default()).map_err(|reason| {
                format!(
                    "{}:{line}: invalid {:?} flag {:?}: {reason}; expected true, false, 1, or 0",
                    path.display(),
                    options.ok_column,
                    record.get(index).unwrap_or_default()
                )
            })?
        } else {
            true
        };

        operations.push(RawOperation {
            latency_us,
            start_seconds,
            ok,
        });
    }

    if operations.is_empty() {
        return Err(format!(
            "{}: empty run: CSV has a header but no data rows",
            path.display()
        ));
    }

    let (lower_bound, upper_bound, window_seconds) = if start_index.is_some() {
        let first_start = operations
            .iter()
            .filter_map(|operation| operation.start_seconds)
            .min_by(f64::total_cmp)
            .expect("start times were parsed for every operation");
        let last_start = operations
            .iter()
            .filter_map(|operation| operation.start_seconds)
            .max_by(f64::total_cmp)
            .expect("start times were parsed for every operation");
        let lower = first_start + options.warmup_seconds.unwrap_or(0.0);
        let upper = last_start - options.cooldown_seconds.unwrap_or(0.0);
        (Some(lower), Some(upper), Some((upper - lower).max(0.0)))
    } else {
        (None, None, None)
    };

    let mut dropped = DropCounts {
        warmup: 0,
        cooldown: 0,
        failed: 0,
    };
    let mut latency_values_us = Vec::new();
    for operation in &operations {
        if let Some(start) = operation.start_seconds {
            if start < lower_bound.expect("start bounds exist") {
                dropped.warmup += 1;
                continue;
            }
            if start > upper_bound.expect("start bounds exist") {
                dropped.cooldown += 1;
                continue;
            }
        }
        if !operation.ok {
            dropped.failed += 1;
            continue;
        }
        latency_values_us.push(operation.latency_us);
    }

    if latency_values_us.is_empty() {
        let trimmed = dropped.warmup + dropped.cooldown;
        if trimmed == operations.len() {
            return Err(format!(
                "{}: empty run after trimming ({} warm-up and {} cool-down operations dropped)",
                path.display(),
                dropped.warmup,
                dropped.cooldown
            ));
        }
        return Err(format!(
            "{}: no successful latency samples remain after trimming and failed-operation exclusion ({} failed operations)",
            path.display(),
            dropped.failed
        ));
    }

    latency_values_us.sort_by(f64::total_cmp);
    let latency = summarize_latencies(&latency_values_us);
    let kept = latency_values_us.len();
    let throughput_ops_per_second = window_seconds
        .filter(|seconds| *seconds > 0.0)
        .map(|seconds| kept as f64 / seconds);

    Ok(RunSummary {
        file: path.display().to_string(),
        operations: OperationCounts {
            total_records: operations.len(),
            kept,
            dropped,
        },
        window_seconds,
        throughput_ops_per_second,
        latency,
        latency_values_us,
    })
}

pub fn percentile_nearest_rank(sorted: &[f64], probability: f64) -> Option<f64> {
    if sorted.is_empty() || !probability.is_finite() || !(0.0..=1.0).contains(&probability) {
        return None;
    }
    let rank = ((probability * sorted.len() as f64).ceil() as usize).max(1);
    sorted
        .get(rank.saturating_sub(1).min(sorted.len() - 1))
        .copied()
}

pub fn summarize_latencies(sorted: &[f64]) -> LatencyStats {
    debug_assert!(!sorted.is_empty());
    let count = sorted.len() as f64;
    let mean_us = sorted.iter().sum::<f64>() / count;
    let variance = sorted
        .iter()
        .map(|value| {
            let difference = value - mean_us;
            difference * difference
        })
        .sum::<f64>()
        / count;
    LatencyStats {
        min_us: sorted[0],
        p50_us: percentile_nearest_rank(sorted, 0.5).expect("non-empty input"),
        p90_us: percentile_nearest_rank(sorted, 0.9).expect("non-empty input"),
        p99_us: percentile_nearest_rank(sorted, 0.99).expect("non-empty input"),
        p99_9_us: percentile_nearest_rank(sorted, 0.999).expect("non-empty input"),
        max_us: *sorted.last().expect("non-empty input"),
        mean_us,
        stddev_us: variance.sqrt(),
    }
}

pub fn across_run_stats(values: &[f64], unit: &str) -> Result<Option<AcrossRunStats>, String> {
    if values.len() < 2 {
        return Ok(None);
    }
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let sample_variance = values
        .iter()
        .map(|value| {
            let difference = value - mean;
            difference * difference
        })
        .sum::<f64>()
        / (n - 1) as f64;
    let sample_stddev = sample_variance.sqrt();
    let distribution = StudentsT::new(0.0, 1.0, (n - 1) as f64)
        .map_err(|error| format!("cannot construct Student t distribution: {error}"))?;
    let t_critical = distribution.inverse_cdf(0.975);
    let margin = t_critical * sample_stddev / (n as f64).sqrt();
    Ok(Some(AcrossRunStats {
        n,
        unit: unit.to_owned(),
        mean,
        sample_stddev,
        ci95_low: mean - margin,
        ci95_high: mean + margin,
    }))
}

pub fn summarize_across_runs(
    runs: &[RunSummary],
) -> Result<Option<BTreeMap<String, AcrossRunStats>>, String> {
    if runs.len() < 2 {
        return Ok(None);
    }
    let first_metrics = metrics_for_run(&runs[0]);
    let mut summary = BTreeMap::new();
    for metric in first_metrics {
        let values: Vec<f64> = runs
            .iter()
            .filter_map(|run| {
                metrics_for_run(run)
                    .into_iter()
                    .find(|candidate| candidate.name == metric.name)
                    .and_then(|candidate| candidate.value)
            })
            .collect();
        if let Some(stats) = across_run_stats(&values, metric.unit)? {
            summary.insert(metric.name.to_owned(), stats);
        }
    }
    Ok(Some(summary))
}

pub fn write_cdf(path: &Path, runs: &[RunSummary], max_points: usize) -> Result<(), String> {
    if max_points < 2 {
        return Err("--cdf-points must be at least 2".to_owned());
    }
    ensure_cdf_path_is_not_input(path, runs)?;
    let mut writer = csv::Writer::from_path(path)
        .map_err(|error| format!("{}: cannot create CDF CSV: {error}", path.display()))?;
    writer
        .write_record(["run", "latency_us", "fraction"])
        .map_err(|error| format!("{}: cannot write CDF header: {error}", path.display()))?;
    for run in runs {
        for (latency_us, fraction) in empirical_cdf_points(&run.latency_values_us, max_points) {
            writer
                .write_record([
                    run.file.as_str(),
                    &latency_us.to_string(),
                    &fraction.to_string(),
                ])
                .map_err(|error| format!("{}: cannot write CDF row: {error}", path.display()))?;
        }
    }
    writer
        .flush()
        .map_err(|error| format!("{}: cannot flush CDF CSV: {error}", path.display()))
}

pub fn empirical_cdf_points(values: &[f64], max_points: usize) -> Vec<(f64, f64)> {
    if values.is_empty() || max_points == 0 {
        return Vec::new();
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mut unique = Vec::new();
    let mut position = 0;
    while position < sorted.len() {
        let value = sorted[position];
        let mut end = position + 1;
        while end < sorted.len() && sorted[end] == value {
            end += 1;
        }
        unique.push((value, end as f64 / sorted.len() as f64));
        position = end;
    }
    if unique.len() <= max_points {
        return unique;
    }
    if max_points == 1 {
        return vec![unique[0]];
    }
    (0..max_points)
        .map(|index| {
            let source_index =
                (index * (unique.len() - 1) + (max_points - 1) / 2) / (max_points - 1);
            unique[source_index]
        })
        .collect()
}

pub fn render_human(runs: &[RunSummary], across: Option<&BTreeMap<String, AcrossRunStats>>) {
    println!("Per-run summary (latency in ms):");
    println!(
        "run | kept | warm-up | cool-down | failed | window(s) | successful ops/s | min | p50 | p90 | p99 | p99.9 | max | mean | sd"
    );
    for run in runs {
        let latency = &run.latency;
        let latency_values = [
            latency.min_us,
            latency.p50_us,
            latency.p90_us,
            latency.p99_us,
            latency.p99_9_us,
            latency.max_us,
            latency.mean_us,
            latency.stddev_us,
        ];
        println!(
            "{} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {}",
            run.file,
            run.operations.kept,
            run.operations.dropped.warmup,
            run.operations.dropped.cooldown,
            run.operations.dropped.failed,
            format_optional(run.window_seconds, 1.0),
            format_optional(run.throughput_ops_per_second, 1.0),
            format_latency_ms(latency_values[0]),
            format_latency_ms(latency_values[1]),
            format_latency_ms(latency_values[2]),
            format_latency_ms(latency_values[3]),
            format_latency_ms(latency_values[4]),
            format_latency_ms(latency_values[5]),
            format_latency_ms(latency_values[6]),
            format_latency_ms(latency_values[7]),
        );
    }
    if let Some(across) = across {
        println!();
        println!("Across-run summaries (mean, sample SD, two-sided 95% Student t CI):");
        println!("metric | n | unit | mean | sample SD | CI low | CI high");
        for (name, stats) in across {
            let display_unit = display_unit(&stats.unit);
            let scale = display_scale(&stats.unit);
            // Latency values are shown in ms here, so drop the `_us` suffix of the JSON key.
            let display_name = name.strip_suffix("_us").unwrap_or(name);
            println!(
                "{} | {} | {} | {} | {} | {} | {}",
                display_name,
                stats.n,
                display_unit,
                format_scaled(stats.mean, scale),
                format_scaled(stats.sample_stddev, scale),
                format_scaled(stats.ci95_low, scale),
                format_scaled(stats.ci95_high, scale),
            );
        }
    }
}

pub fn make_json_report(unit: Unit, runs: Vec<RunSummary>) -> Result<JsonReport, String> {
    let across_runs = summarize_across_runs(&runs)?;
    Ok(JsonReport {
        input_unit: unit.as_str().to_owned(),
        latency_unit: "us",
        window_unit: "seconds",
        throughput_unit: "successful_operations_per_second",
        runs,
        across_runs,
    })
}

pub fn validate_duration(name: &str, duration: Option<f64>) -> Result<(), String> {
    if let Some(value) = duration
        && (!value.is_finite() || value < 0.0)
    {
        return Err(format!(
            "{name} must be a finite, non-negative number of seconds"
        ));
    }
    Ok(())
}

fn column_index(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    headers.iter().position(|header| header == name)
}

fn parse_number(text: &str) -> Result<f64, &'static str> {
    let value = text
        .trim()
        .parse::<f64>()
        .map_err(|_| "expected a number")?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err("expected a finite number")
    }
}

fn parse_ok(text: &str) -> Result<bool, &'static str> {
    match text.trim().to_ascii_lowercase().as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err("unrecognized value"),
    }
}

fn csv_error_line(error: &csv::Error) -> Option<u64> {
    error.position().map(|position| position.line())
}

fn summarize_metrics(stats: &LatencyStats) -> [f64; 8] {
    [
        stats.min_us,
        stats.p50_us,
        stats.p90_us,
        stats.p99_us,
        stats.p99_9_us,
        stats.max_us,
        stats.mean_us,
        stats.stddev_us,
    ]
}

fn metrics_for_run(run: &RunSummary) -> Vec<MetricValue> {
    let latency = summarize_metrics(&run.latency);
    vec![
        MetricValue {
            name: "total_records",
            value: Some(run.operations.total_records as f64),
            unit: "records",
        },
        MetricValue {
            name: "kept_successful_operations",
            value: Some(run.operations.kept as f64),
            unit: "operations",
        },
        MetricValue {
            name: "dropped_warmup_operations",
            value: Some(run.operations.dropped.warmup as f64),
            unit: "operations",
        },
        MetricValue {
            name: "dropped_cooldown_operations",
            value: Some(run.operations.dropped.cooldown as f64),
            unit: "operations",
        },
        MetricValue {
            name: "dropped_failed_operations",
            value: Some(run.operations.dropped.failed as f64),
            unit: "operations",
        },
        MetricValue {
            name: "window_seconds",
            value: run.window_seconds,
            unit: "s",
        },
        MetricValue {
            name: "throughput_ops_per_second",
            value: run.throughput_ops_per_second,
            unit: "ops/s",
        },
        MetricValue {
            name: "latency_min_us",
            value: Some(latency[0]),
            unit: "us",
        },
        MetricValue {
            name: "latency_p50_us",
            value: Some(latency[1]),
            unit: "us",
        },
        MetricValue {
            name: "latency_p90_us",
            value: Some(latency[2]),
            unit: "us",
        },
        MetricValue {
            name: "latency_p99_us",
            value: Some(latency[3]),
            unit: "us",
        },
        MetricValue {
            name: "latency_p99_9_us",
            value: Some(latency[4]),
            unit: "us",
        },
        MetricValue {
            name: "latency_max_us",
            value: Some(latency[5]),
            unit: "us",
        },
        MetricValue {
            name: "latency_mean_us",
            value: Some(latency[6]),
            unit: "us",
        },
        MetricValue {
            name: "latency_stddev_us",
            value: Some(latency[7]),
            unit: "us",
        },
    ]
}

fn ensure_cdf_path_is_not_input(path: &Path, runs: &[RunSummary]) -> Result<(), String> {
    let output = fs::canonicalize(path).ok();
    for run in runs {
        let input = fs::canonicalize(PathBuf::from(&run.file)).ok();
        if output.is_some() && output == input {
            return Err(format!(
                "CDF output {} must not overwrite an input file",
                path.display()
            ));
        }
    }
    Ok(())
}

fn display_unit(unit: &str) -> &str {
    if unit == "us" { "ms" } else { unit }
}

fn display_scale(unit: &str) -> f64 {
    if unit == "us" { 0.001 } else { 1.0 }
}

fn format_optional(value: Option<f64>, scale: f64) -> String {
    value
        .map(|number| format_scaled(number, scale))
        .unwrap_or_else(|| "-".to_owned())
}

fn format_latency_ms(value_us: f64) -> String {
    format_scaled(value_us, 0.001)
}

fn format_scaled(value: f64, scale: f64) -> String {
    format!("{:.3}", value * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn nearest_rank_handles_single_and_two_values() {
        assert_eq!(percentile_nearest_rank(&[7.0], 0.5), Some(7.0));
        assert_eq!(percentile_nearest_rank(&[10.0, 20.0], 0.5), Some(10.0));
        assert_eq!(percentile_nearest_rank(&[10.0, 20.0], 0.9), Some(20.0));
    }

    #[test]
    fn nearest_rank_uses_exact_one_based_ranks() {
        let values = [10.0, 20.0, 30.0, 40.0];
        assert_eq!(percentile_nearest_rank(&values, 0.25), Some(10.0));
        assert_eq!(percentile_nearest_rank(&values, 0.5), Some(20.0));
        assert_eq!(percentile_nearest_rank(&values, 0.75), Some(30.0));
        assert_eq!(percentile_nearest_rank(&values, 1.0), Some(40.0));
        assert_eq!(percentile_nearest_rank(&values, -0.1), None);
    }

    #[test]
    fn warmup_cooldown_and_failed_operations_are_counted_separately() {
        let options = ProcessOptions {
            warmup_seconds: Some(1.0),
            cooldown_seconds: Some(1.0),
            ..ProcessOptions::default()
        };
        let run = process_file(&fixture_path("trim.csv"), &options).unwrap();
        assert_eq!(run.operations.total_records, 5);
        assert_eq!(run.operations.kept, 2);
        assert_eq!(run.operations.dropped.warmup, 1);
        assert_eq!(run.operations.dropped.cooldown, 1);
        assert_eq!(run.operations.dropped.failed, 1);
        assert_eq!(run.window_seconds, Some(2.0));
        assert_eq!(run.throughput_ops_per_second, Some(1.0));
        assert_eq!(run.latency.min_us, 200.0);
        assert_eq!(run.latency.max_us, 400.0);
        assert_eq!(run.latency.mean_us, 300.0);
    }

    #[test]
    fn student_t_critical_values_match_reference_values() {
        let df4 = StudentsT::new(0.0, 1.0, 4.0).unwrap().inverse_cdf(0.975);
        let df9 = StudentsT::new(0.0, 1.0, 9.0).unwrap().inverse_cdf(0.975);
        assert!((df4 - 2.776).abs() < 0.001);
        assert!((df9 - 2.262).abs() < 0.001);
    }

    #[test]
    fn t_based_interval_matches_hand_computation() {
        let stats = across_run_stats(&[1.0, 2.0, 3.0, 4.0, 5.0], "x")
            .unwrap()
            .unwrap();
        assert!((stats.mean - 3.0).abs() < 1e-12);
        assert!((stats.sample_stddev - 2.5_f64.sqrt()).abs() < 1e-12);
        assert!((stats.ci95_low - 1.037).abs() < 0.002);
        assert!((stats.ci95_high - 4.963).abs() < 0.002);
    }

    #[test]
    fn csv_errors_include_file_and_line() {
        let error =
            process_file(&fixture_path("bad_latency.csv"), &ProcessOptions::default()).unwrap_err();
        assert!(error.contains("bad_latency.csv"));
        assert!(error.contains(":3:"));
        assert!(error.contains("invalid latency"));
    }

    #[test]
    fn warmup_requires_start_column() {
        let options = ProcessOptions {
            warmup_seconds: Some(0.0),
            ..ProcessOptions::default()
        };
        let error = process_file(&fixture_path("no_start.csv"), &options).unwrap_err();
        assert!(error.contains("--warmup requires start column"));
    }

    #[test]
    fn cdf_downsampling_keeps_extremes() {
        let points = empirical_cdf_points(&[1.0, 2.0, 3.0, 4.0, 5.0], 3);
        assert_eq!(points.len(), 3);
        assert_eq!(points.first().unwrap().0, 1.0);
        assert_eq!(points.last().unwrap().0, 5.0);
        assert_eq!(points.last().unwrap().1, 1.0);
    }
}
