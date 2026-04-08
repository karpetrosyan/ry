use pretty_assertions::assert_eq;
use ry::config::Config;
use ry::inline::process_file_inline;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

fn get_fixture_dirs() -> Vec<PathBuf> {
    let fixtures_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");

    let mut dirs = Vec::new();

    if let Ok(entries) = fs::read_dir(&fixtures_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                dirs.push(path);
            }
        }
    }

    dirs.sort();
    dirs
}

fn run_fixture_test(fixture_dir: &Path) {
    let fixture_name = fixture_dir.file_name().unwrap().to_string_lossy();

    let config_path = fixture_dir.join("ry.yml");
    let input_path = fixture_dir.join("input.py");
    let expected_path = fixture_dir.join("expected.py");

    assert!(
        config_path.exists(),
        "Fixture '{}' missing ry.yml",
        fixture_name
    );
    assert!(
        input_path.exists(),
        "Fixture '{}' missing input.py",
        fixture_name
    );

    let temp_dir = TempDir::new().unwrap();
    let temp_input = temp_dir.path().join("input.py");

    fs::copy(&input_path, &temp_input).unwrap();

    let config = Config::from_file(config_path.to_str().unwrap()).unwrap_or_else(|e| {
        panic!(
            "Failed to load config for fixture '{}': {}",
            fixture_name, e
        )
    });

    let result = process_file_inline(&temp_input, &config, false);

    assert!(
        result.is_ok(),
        "Fixture '{}' transformation failed: {:?}",
        fixture_name,
        result.err()
    );

    let actual_content = fs::read_to_string(&temp_input).unwrap();

    let update_fixtures = std::env::var("UPDATE_FIXTURES").is_ok();

    if update_fixtures || !expected_path.exists() {
        fs::write(&expected_path, &actual_content).unwrap();
        println!("Fixture '{}': Generated expected.py", fixture_name);
    } else {
        let expected_content = fs::read_to_string(&expected_path).unwrap();
        assert_eq!(
            actual_content, expected_content,
            "Fixture '{}': Output does not match expected",
            fixture_name
        );
    }
}

#[test]
fn test_all_fixtures() {
    let fixture_dirs = get_fixture_dirs();

    assert!(
        !fixture_dirs.is_empty(),
        "No fixtures found in tests/fixtures/"
    );

    for fixture_dir in fixture_dirs {
        println!("Running fixture: {}", fixture_dir.display());
        run_fixture_test(&fixture_dir);
    }
}
