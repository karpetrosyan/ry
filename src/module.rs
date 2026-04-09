use crate::apply_rules;
use crate::config::{Config, TargetVariant};
use crate::diagnostics::Diagnostic;
use std::fs;

pub fn process_all_file_targets(
    config: &Config,
    check_only: bool,
) -> Result<(Vec<Diagnostic>, Vec<Diagnostic>), String> {
    let mut diagnostics = Vec::new();
    let mut fixed_diagnostics = Vec::new();

    for target in &config.targets {
        if let TargetVariant::File { input, output } = &target.variant {
            let rules = config.get_effective_rules(target);

            let source_code = fs::read_to_string(input)
                .map_err(|e| format!("Error reading input file '{}': {}", input, e))?;

            let transformed = apply_rules(&source_code, &rules);

            let needs_update = match fs::read_to_string(output) {
                Ok(existing) => existing != transformed,
                Err(_) => true,
            };

            if needs_update {
                let diagnostic = Diagnostic::error(
                    input,
                    1,
                    0,
                    format!("Output file '{}' is out of sync with input", output),
                    crate::diagnostics::DiagnosticKind::OutOfSyncModuleTarget,
                );

                if check_only {
                    diagnostics.push(diagnostic);
                } else {
                    fs::write(output, transformed)
                        .map_err(|e| format!("Error writing output file '{}': {}", output, e))?;

                    fixed_diagnostics.push(diagnostic);
                }
            }
        }
    }

    Ok((diagnostics, fixed_diagnostics))
}
