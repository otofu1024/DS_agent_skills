#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::Parser;
use latstat::{
    JsonReport, ProcessOptions, Unit, make_json_report, process_file, render_human,
    summarize_across_runs, write_cdf,
};

#[derive(Debug, Parser)]
#[command(
    name = "latstat",
    version,
    about = "Summarize latency benchmark CSVs as independent runs",
    after_help = "Percentiles use nearest rank: sort the retained values and select rank max(1, ceil(p * n)), with rank numbered from 1."
)]
struct Cli {
    /// One CSV file per independent run.
    #[arg(required = true, value_name = "CSV")]
    files: Vec<PathBuf>,

    /// Required latency column.
    #[arg(long, default_value = "latency_us")]
    latency_column: String,

    /// Optional start-time column, used for trimming and throughput when present.
    #[arg(long, default_value = "start_us")]
    start_column: String,

    /// Optional success column; accepted values are true, false, 1, and 0.
    #[arg(long, default_value = "ok")]
    ok_column: String,

    /// Unit shared by latency and start-time values in the input files.
    #[arg(long, value_enum, default_value = "us")]
    unit: Unit,

    /// Drop operations earlier than first start + SECONDS.
    #[arg(long, value_name = "SECONDS")]
    warmup: Option<f64>,

    /// Drop operations later than last start - SECONDS.
    #[arg(long, value_name = "SECONDS")]
    cooldown: Option<f64>,

    /// Emit machine-readable JSON instead of the human-readable tables.
    #[arg(long)]
    json: bool,

    /// Write a combined empirical CDF CSV to PATH.
    #[arg(long, value_name = "PATH")]
    cdf: Option<PathBuf>,

    /// Maximum empirical-CDF points per run (minimum 2; endpoints are retained).
    #[arg(long, default_value_t = 1000, value_name = "POINTS")]
    cdf_points: usize,
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = execute(cli) {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn execute(cli: Cli) -> Result<(), String> {
    if cli.cdf.is_some() && cli.cdf_points < 2 {
        return Err("--cdf-points must be at least 2".to_owned());
    }

    let options = ProcessOptions {
        latency_column: cli.latency_column,
        start_column: cli.start_column,
        ok_column: cli.ok_column,
        unit: cli.unit,
        warmup_seconds: cli.warmup,
        cooldown_seconds: cli.cooldown,
    };
    let mut runs = Vec::with_capacity(cli.files.len());
    for path in &cli.files {
        runs.push(process_file(path, &options)?);
    }

    if runs.len() < 5 {
        eprintln!(
            "warning: {} independent run(s) provided; fewer than 5 runs may give imprecise across-run confidence intervals",
            runs.len()
        );
    }

    if let Some(path) = cli.cdf.as_deref() {
        write_cdf(path, &runs, cli.cdf_points)?;
    }

    if cli.json {
        let report: JsonReport = make_json_report(cli.unit, runs)?;
        let json = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("cannot serialize JSON report: {error}"))?;
        println!("{json}");
    } else {
        let across = summarize_across_runs(&runs)?;
        render_human(&runs, across.as_ref());
    }
    Ok(())
}
