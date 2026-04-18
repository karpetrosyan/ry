use crate::apply_rules;
use crate::config::{Config, Rule, TargetVariant};
use crate::diagnostics::{Diagnostic, DiagnosticKind};
use crate::treesitter::{get_node_text, PythonParser};
use regex::Regex;
use std::fs;
use std::path::Path;
use tree_sitter::Node;

enum Mark {
    Generate(Option<String>),
    Generated(Option<String>),
}

fn extract_trailing_comment(source: &str) -> Option<Mark> {
    let lines: Vec<&str> = source.lines().collect();
    if lines.len() == 0 {
        return None;
    }

    let line_text = lines[0];
    let generate_re = Regex::new(r"#\s*unasync:\s*generate(?:\s+@([\w\-]+))?\s*$").unwrap();
    let generated_re = Regex::new(r"#\s*unasync:\s*generated(?:\s+@([\w\-]+))?\s*$").unwrap();

    if let Some(caps) = generate_re.captures(line_text) {
        let tag = caps.get(1).map(|m| m.as_str().to_string());
        return Some(Mark::Generate(tag));
    }

    if let Some(caps) = generated_re.captures(line_text) {
        let tag = caps.get(1).map(|m| m.as_str().to_string());
        return Some(Mark::Generated(tag));
    }

    None
}

/// Processes a single Python file for inline generation, returning all diagnostics and fixed diagnostics.
pub fn process_file_inline(
    file_path: &Path,
    config: &Config,
    check_only: bool,
) -> Result<(Vec<Diagnostic>, Vec<Diagnostic>), String> {
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut parser = PythonParser::new()?;

    let tree = parser
        .parse(&content)
        .ok_or("Failed to parse Python code")?;

    let file_str = file_path.to_string_lossy();

    let diagnostics = validate_all_inline(&tree.root_node(), config, &content, &file_str);

    if check_only {
        return Ok((diagnostics, Vec::new()));
    }

    let fixable_diagnostics: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.is_fixable())
        .cloned()
        .collect();

    if fixable_diagnostics.is_empty() {
        return Ok((diagnostics, Vec::new()));
    }

    let modified_content = process_fixable_diagnostics(&diagnostics, &content);

    if modified_content != content {
        fs::write(file_path, modified_content)
            .map_err(|e| format!("Failed to write modified content to file: {}", e))?;

        let new_diagnostics = validate_all_inline(
            &parser
                .parse(
                    &fs::read_to_string(file_path)
                        .map_err(|e| format!("Failed to read file: {}", e))?,
                )
                .ok_or("Failed to parse file")?
                .root_node(),
            config,
            &fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?,
            &file_str,
        );

        return Ok((new_diagnostics, fixable_diagnostics));
    }

    Ok((diagnostics, Vec::new()))
}

/// Processes a list of generate operations by splicing transformed code into the original content,
/// returning the modified content and a list of generations.
fn process_fixable_diagnostics(diagnostics: &[Diagnostic], original_content: &str) -> String {
    let mut result = String::new();
    let mut last_pos = 0;

    for diagnostic in diagnostics {
        let (generate_end_byte, generated_end_byte, transformed_code, indentation) =
            match &diagnostic.kind {
                DiagnosticKind::MissingInlineGeneration {
                    generate_end_byte,
                    generated_end_byte,
                    transformed_code,
                    indentation,
                    ..
                } => (
                    *generate_end_byte,
                    *generated_end_byte,
                    transformed_code,
                    *indentation,
                ),
                DiagnosticKind::OutDatedGeneratedCode {
                    generate_end_byte,
                    generated_end_byte,
                    transformed_code,
                    indentation,
                    ..
                } => (
                    *generate_end_byte,
                    *generated_end_byte,
                    transformed_code,
                    *indentation,
                ),
                _ => continue,
            };

        result.push_str(&original_content[last_pos..generate_end_byte]);

        result.push_str("\n\n");
        result.push_str(" ".repeat(indentation).as_str());
        result.push_str(transformed_code);

        if let Some(generated_end) = generated_end_byte {
            last_pos = generated_end;
        } else {
            last_pos = generate_end_byte;
        }
    }

    result.push_str(&original_content[last_pos..]);
    result
}

fn find_target_by_tag<'a>(config: &'a Config, tag: &str) -> Option<&'a crate::config::Target> {
    config
        .targets
        .iter()
        .find(|target| matches!(&target.variant, TargetVariant::Inline { tag: t } if t == tag))
}

fn validate_all_inline(node: &Node, config: &Config, source: &str, file: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    validate_all_inline_recursive(node, config, source, file, &mut diagnostics);
    diagnostics
}

fn validate_all_inline_recursive(
    node: &Node,
    config: &Config,
    source: &str,
    file: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if is_definition_node(node.kind()) {
        let node_text = get_node_text(node, source);
        let start_line = node.start_position().row;

        if let Some(Mark::Generate(tag)) = extract_trailing_comment(node_text) {
            let rules = if let Some(tag_name) = tag.as_deref() {
                match find_target_by_tag(config, tag_name) {
                    Some(target) => config.get_effective_rules(target),
                    None => {
                        let col = node.start_position().column;
                        diagnostics.push(Diagnostic::warning(
                            file,
                            start_line + 1,
                            col,
                            format!(
                                "Tag '@{}' used in marker but not found in config targets",
                                tag_name
                            ),
                            DiagnosticKind::TagUsedInMarkerNotFoundInConfig {
                                tag_name: tag_name.to_string(),
                            },
                        ));
                        return;
                    }
                }
            } else {
                config.get_effective_rules_from_defaults()
            };

            if let Some(d) = validate_generate_node(node, &tag, &rules, source, file) {
                diagnostics.push(d);
            }
        } else if let Some(Mark::Generated(tag)) = extract_trailing_comment(node_text) {
            if let Some(d) = validate_generated_node(node, &tag, source, file) {
                diagnostics.push(d);
            }
        }
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        validate_all_inline_recursive(&child, config, source, file, diagnostics);
    }
}

fn is_definition_node(kind: &str) -> bool {
    matches!(
        kind,
        "function_definition" | "async_function_definition" | "class_definition"
    )
}

fn validate_generate_node(
    node: &Node,
    tag: &Option<String>,
    rules: &[Rule],
    source: &str,
    file: &str,
) -> Option<Diagnostic> {
    let start_line = node.start_position().row;
    let col = node.start_position().column;
    let tag_suffix = tag.as_ref().map(|t| format!(" @{}", t)).unwrap_or_default();

    let generate_node_text = get_node_text(node, source);
    let mut transformed = apply_rules(generate_node_text, rules);

    let generate_re = Regex::new(r"#\s*unasync:\s*generate\b").unwrap();
    transformed = generate_re
        .replace(&transformed, "# unasync: generated")
        .to_string();

    let indentation = node.start_position().column;
    let generate_end_byte = node.end_byte();

    let next_sibling = node.next_sibling();

    if let Some(next) = next_sibling {
        if !is_definition_node(next.kind()) {
            return Some(Diagnostic::error(
                file,
                start_line + 1,
                col,
                format!("Generate marker{} must be followed by a definition node with a generated marker", tag_suffix),
                DiagnosticKind::MissingInlineGeneration {
                    generate_end_byte,
                    generated_end_byte: None,
                    transformed_code: transformed,
                    indentation,
                    source_line: start_line + 1,
                },
            ));
        }

        let next_text = get_node_text(&next, source);
        if let Some(Mark::Generated(next_tag)) = extract_trailing_comment(next_text) {
            if tag != &next_tag {
                return Some(Diagnostic::error(
                    file,
                    start_line + 1,
                    col,
                    format!(
                        "Generate marker{} tag does not match generated marker tag{}",
                        tag_suffix,
                        next_tag
                            .as_ref()
                            .map(|t| format!(" @{}", t))
                            .unwrap_or_default()
                    ),
                    DiagnosticKind::MissingInlineGeneration {
                        generate_end_byte,
                        generated_end_byte: Some(next.end_byte()),
                        transformed_code: transformed,
                        indentation,
                        source_line: start_line + 1,
                    },
                ));
            }

            if next_text != transformed {
                return Some(Diagnostic::error(
                    file,
                    start_line + 1,
                    col,
                    format!(
                        "Generated code{} is out of sync with generate marker",
                        tag_suffix
                    ),
                    DiagnosticKind::OutDatedGeneratedCode {
                        generate_end_byte,
                        generated_end_byte: Some(next.end_byte()),
                        transformed_code: transformed,
                        actual_code: next_text.to_string(),
                        indentation,
                        source_line: start_line + 1,
                    },
                ));
            }

            return None;
        } else {
            return Some(Diagnostic::error(
                file,
                start_line + 1,
                col,
                format!(
                    "Generate marker{} must be followed by a generated marker",
                    tag_suffix
                ),
                DiagnosticKind::MissingInlineGeneration {
                    generate_end_byte,
                    generated_end_byte: None,
                    transformed_code: transformed,
                    indentation,
                    source_line: start_line + 1,
                },
            ));
        }
    } else {
        return Some(Diagnostic::error(
            file,
            start_line + 1,
            col,
            format!(
                "Generate marker{} must be followed by a generated node",
                tag_suffix
            ),
            DiagnosticKind::MissingInlineGeneration {
                generate_end_byte,
                generated_end_byte: None,
                transformed_code: transformed,
                indentation,
                source_line: start_line + 1,
            },
        ));
    }
}

fn validate_generated_node(
    node: &Node,
    tag: &Option<String>,
    source: &str,
    file: &str,
) -> Option<Diagnostic> {
    let start_line = node.start_position().row;
    let col = node.start_position().column;
    let tag_suffix = tag.as_ref().map(|t| format!(" @{}", t)).unwrap_or_default();

    let prev_sibling = node.prev_sibling()?;

    if !is_definition_node(prev_sibling.kind()) {
        return Some(Diagnostic::error(
            file,
            start_line + 1,
            col,
            format!(
                "Orphaned generated marker{} without matching generate marker above",
                tag_suffix
            ),
            DiagnosticKind::GeneratedCodeWithoutGenerator,
        ));
    }

    let prev_text = get_node_text(&prev_sibling, source);
    if let Some(Mark::Generate(prev_tag)) = extract_trailing_comment(prev_text) {
        if &prev_tag != tag {
            return Some(Diagnostic::error(
                file,
                start_line + 1,
                col,
                format!(
                    "Generated marker{} tag does not match generate marker tag{}",
                    tag_suffix,
                    prev_tag
                        .as_ref()
                        .map(|t| format!(" @{}", t))
                        .unwrap_or_default()
                ),
                DiagnosticKind::GeneratedCodeWithoutGenerator,
            ));
        }
    } else {
        return Some(Diagnostic::error(
            file,
            start_line + 1,
            col,
            format!(
                "Orphaned generated marker{} without matching generate marker above",
                tag_suffix
            ),
            DiagnosticKind::GeneratedCodeWithoutGenerator,
        ));
    }

    None
}
