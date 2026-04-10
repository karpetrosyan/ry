pub mod config;
pub mod diagnostics;
pub mod generation;
pub mod inline;
pub mod module;
pub mod packages;
pub mod treesitter;

use config::{Rule, RuleKind};
use regex::Regex;

pub fn apply_rules(source: &str, rules: &[Rule]) -> String {
    let stripped: String = source
        .lines()
        .filter(|line| !line.contains("# unasync: strip"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut result = if source.ends_with('\n') {
        stripped + "\n"
    } else {
        stripped
    };

    for rule in rules {
        match rule.kind {
            RuleKind::Regex => {
                if let Ok(re) = Regex::new(&rule.match_pattern) {
                    result = re.replace_all(&result, rule.replace.as_str()).to_string();
                } else {
                    eprintln!(
                        "Warning: Invalid regex pattern in rule '{}': {}",
                        rule.id.as_deref().unwrap_or("<unnamed>"),
                        rule.match_pattern
                    );
                }
            }
        }
    }

    result
}
