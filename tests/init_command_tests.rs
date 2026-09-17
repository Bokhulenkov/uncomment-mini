use std::fs;
use tempfile::TempDir;
use uncomment::cli::Cli;

#[test]
fn test_init_command_basic() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join(".uncommentrc.toml");

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(temp_dir.path()).unwrap();

    let result = Cli::handle_init_command(&output_path, false, false, false);

    std::env::set_current_dir(original_dir).unwrap();

    assert!(result.is_ok());
    assert!(output_path.exists());

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(content.contains("[global]"));
    assert!(content.contains("remove_todos = false"));
    assert!(content.contains("preserve_patterns = [\"HACK\", \"WORKAROUND\", \"NOTE\"]"));
}

#[test]
fn test_init_command_force() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join(".uncommentrc.toml");

    fs::write(&output_path, "existing content").unwrap();

    let result = Cli::handle_init_command(&output_path, false, false, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));

    let result = Cli::handle_init_command(&output_path, true, false, false);
    assert!(result.is_ok());

    let content = fs::read_to_string(&output_path).unwrap();
    assert!(!content.contains("existing content"));
    assert!(content.contains("[global]"));
}

#[test]
fn test_init_command_comprehensive() {
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("comprehensive.toml");

    let result = Cli::handle_init_command(&output_path, false, true, false);
    assert!(result.is_ok());
    assert!(output_path.exists());

    let content = fs::read_to_string(&output_path).unwrap();

    assert!(content.contains("[languages.c]"));
    assert!(content.contains("[languages.cpp]"));
    assert!(content.contains("[languages.objc]"));
    assert!(content.contains("[languages.swift]"));

    assert!(!content.contains("[languages.swift.grammar]"));

    assert!(!content.contains("# Web Development Languages"));
    assert!(!content.contains("# Mobile Development"));
}

#[test]
fn test_init_command_invalid_path() {
    let invalid_path = std::path::PathBuf::from("/nonexistent/path/config.toml");

    let result = Cli::handle_init_command(&invalid_path, false, false, false);
    assert!(result.is_err());
}
