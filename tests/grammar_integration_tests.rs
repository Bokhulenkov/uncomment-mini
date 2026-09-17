use std::fs;
use tempfile::TempDir;
use uncomment::{config::ConfigManager, processor::Processor};

#[test]
fn test_processor_with_swift() {
    let temp_dir = TempDir::new().unwrap();
    let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
    let mut processor = Processor::new();

    let test_file = temp_dir.path().join("test.swift");
    let test_content = r#"
// This is a Swift comment
func hello() {
    /* Block comment */
    print("Hello, Swift!")
}
"#;
    fs::write(&test_file, test_content).unwrap();

    let result = processor.process_file_with_config(&test_file, &config_manager, None);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert!(!processed.processed_content.contains("This is a Swift comment"));
    assert!(!processed.processed_content.contains("Block comment"));
    assert!(processed.processed_content.contains("print(\"Hello, Swift!\")"));
}

#[test]
fn test_processor_with_objc() {
    let temp_dir = TempDir::new().unwrap();
    let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
    let mut processor = Processor::new();

    let test_file = temp_dir.path().join("test.m");
    let test_content = r#"
// This is an Objective-C comment
@implementation Example
- (void)hello {
    /* Block comment */
    NSLog(@"Hello, Objective-C!");
}
@end
"#;
    fs::write(&test_file, test_content).unwrap();

    let result = processor.process_file_with_config(&test_file, &config_manager, None);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert!(!processed.processed_content.contains("This is an Objective-C comment"));
    assert!(!processed.processed_content.contains("Block comment"));
    assert!(processed.processed_content.contains("NSLog(@\"Hello, Objective-C!\");"));
}

#[test]
fn test_processor_unsupported_file_type() {
    let temp_dir = TempDir::new().unwrap();
    let config_manager = ConfigManager::new(temp_dir.path()).unwrap();
    let mut processor = Processor::new();

    let unsupported_file = temp_dir.path().join("test.unknown");
    fs::write(&unsupported_file, "// Some comment").unwrap();

    let result = processor.process_file_with_config(&unsupported_file, &config_manager, None);
    assert!(result.is_err());

    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("Unsupported file type"));
}
