use clap::Parser;
use glob::glob;
use ry::config::Config;
use ry::diagnostics::{Diagnostic, DiagnosticKind, Severity};
use ry::inline::process_file_inline;
use ry::module::process_all_file_targets;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "ry")]
#[command(about = "Python tool using tree-sitter", long_about = None)]
struct Cli {
    #[arg(short, long, default_value = "ry.yml", help = "Path to config file")]
    config: String,

    #[arg(short, long, help = "Show detailed output for each generation")]
    verbose: bool,

    #[arg(long, help = "Apply fixes instead of just checking")]
    fix: bool,

    #[arg(default_value = ".", help = "Paths to search for Python files")]
    paths: Vec<String>,
}

fn grab_effective_paths(input_path: &Path) -> Result<Vec<PathBuf>, String> {
    if input_path.is_file() {
        if input_path.extension().map_or(false, |ext| ext == "py") {
            Ok(vec![input_path.to_path_buf()])
        } else {
            Err(format!("'{}' is not a Python file", input_path.display()))
        }
    } else {
        let pattern = format!(
            "{}/**/*.py",
            input_path.to_string_lossy().trim_end_matches('/')
        );

        let matches = glob(&pattern)
            .map_err(|e| format!("Error parsing glob pattern '{}': {}", pattern, e))?;

        let mut paths = Vec::new();
        for entry in matches {
            match entry {
                Ok(path) => paths.push(path),
                Err(e) => return Err(format!("Error reading glob entry: {}", e)),
            }
        }
        Ok(paths)
    }
}

fn main() {
    let cli = Cli::parse();

    let validate = !cli.fix;

    let config = match Config::from_file(&cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error loading config file '{}': {}", cli.config, e);
            std::process::exit(1);
        }
    };

    let mut paths_to_process = Vec::new();

    for path in &cli.paths {
        let input_path = Path::new(path);
        match grab_effective_paths(input_path) {
            Ok(mut paths) => paths_to_process.append(&mut paths),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    let mut diagnostics = Vec::new();
    let mut fixed_diagnostics = Vec::new();

    for path in &paths_to_process {
        let (inline_diagnostics, inline_fixed) = process_file_inline(&path, &config, validate)
            .unwrap_or_else(|e| {
                eprintln!("Error processing file '{}': {}", path.display(), e);
                std::process::exit(1);
            });

        for diagnostic in &inline_diagnostics {
            eprintln!("{}", diagnostic);
        }
        diagnostics.extend(inline_diagnostics);
        fixed_diagnostics.extend(inline_fixed);
    }

    let (files_diagnostics, files_fixed) = process_all_file_targets(&config, validate)
        .unwrap_or_else(|e| {
            eprintln!("Error processing module targets: {}", e);
            std::process::exit(1);
        });

    for diagnostic in &files_diagnostics {
        eprintln!("{}", diagnostic);
    }
    diagnostics.extend(files_diagnostics);
    fixed_diagnostics.extend(files_fixed);

    print_summary(&paths_to_process, &diagnostics, &fixed_diagnostics, cli.verbose);

    if diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error))
    {
        std::process::exit(1)
    }
}

fn print_diff(actual: &str, expected: &str) {
    use similar::{ChangeTag, TextDiff};
    use std::io::IsTerminal;
    let colors = std::io::stderr().is_terminal();
    let diff = TextDiff::from_lines(actual, expected);
    for change in diff.iter_all_changes() {
        let (sign, color, reset) = match change.tag() {
            ChangeTag::Delete => ("-", "\x1b[31m", "\x1b[0m"),
            ChangeTag::Insert => ("+", "\x1b[32m", "\x1b[0m"),
            ChangeTag::Equal => (" ", "", ""),
        };
        if colors {
            eprint!("{}{}{}{}", color, sign, change, reset);
        } else {
            eprint!("{}{}", sign, change);
        }
    }
}

fn print_summary(
    paths_to_process: &[PathBuf],
    diagnostics: &[Diagnostic],
    fixed_diagnostics: &[Diagnostic],
    verbose: bool,
) {
    let error_count = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Error))
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| matches!(d.severity, Severity::Warning))
        .count();
    let fixed_count = fixed_diagnostics.len();

    if verbose {
        for diagnostic in diagnostics {
            if let DiagnosticKind::OutDatedGeneratedCode { actual_code, transformed_code, .. } = &diagnostic.kind {
                eprintln!("diff for {}:{}:", diagnostic.location.file, diagnostic.location.line);
                print_diff(actual_code, transformed_code);
            }
        }
    }

    if fixed_count > 0 {
        println!(
            "Processed {} file(s) with {} error(s), {} warning(s), and fixed {} diagnostic(s)",
            paths_to_process.len(),
            error_count,
            warning_count,
            fixed_count
        );
    } else {
        println!(
            "Processed {} file(s) with {} error(s) and {} warning(s)",
            paths_to_process.len(),
            error_count,
            warning_count
        );
    }
}
