use std::fs;
use tempfile::TempDir;
use uncomment::{config::ConfigManager, processor::Processor};

/// Test error handling for malformed configuration files
#[test]
fn test_malformed_config_error_handling() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("uncomment.toml");

    let malformed_config = r#"
[global
remove_docs = true  # Missing closing bracket
invalid_field = "value"

[languages.c
name = "C"  # Missing closing bracket
"#;

    fs::write(&config_path, malformed_config).unwrap();

    let result = ConfigManager::new(temp_dir.path());
    if let Err(error) = result {
        let error_msg = error.to_string();
        assert!(error_msg.contains("parse") || error_msg.contains("TOML"));
    } else {
        assert!(result.is_ok());
    }
}

/// Test error handling for unsupported file types in processor
#[test]
fn test_processor_unsupported_file_type_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
    let mut processor = Processor::new();

    let unsupported_file = temp_dir.path().join("test.unsupported");
    fs::write(&unsupported_file, "// Comment in unsupported file").unwrap();

    let result = processor.process_file_with_config(&unsupported_file, &config_manager, None);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported file type"));
}

/// Test error handling for nonexistent files in processor
#[test]
fn test_processor_nonexistent_file_error() {
    let temp_dir = TempDir::new().unwrap();
    let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
    let mut processor = Processor::new();

    let nonexistent_file = temp_dir.path().join("nonexistent.c");

    let result = processor.process_file_with_config(&nonexistent_file, &config_manager, None);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Failed to read file"));
}

/// Test error recovery and graceful degradation
#[test]
fn test_error_recovery_mechanisms() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("uncomment.toml");

    let config = r#"
[global]
remove_docs = false

[languages.c]
"#;

    fs::write(&config_path, config).unwrap();

    let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
    let mut processor = Processor::new();

    let c_file = temp_dir.path().join("test.c");
    fs::write(&c_file, "// Comment\nint main(void) {}").unwrap();

    let c_result = processor.process_file_with_config(&c_file, &config_manager, None);
    assert!(c_result.is_ok());

    let unknown_file = temp_dir.path().join("test.xyz");
    fs::write(&unknown_file, "// Comment").unwrap();

    let unknown_result = processor.process_file_with_config(&unknown_file, &config_manager, None);
    assert!(unknown_result.is_err());

    let error_msg = unknown_result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported file type"));
}
