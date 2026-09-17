use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_gitignore_from_subdirectory() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let subfolder = root.join("subfolder");
    let next_folder = subfolder.join(".next");

    fs::create_dir(&subfolder).unwrap();
    fs::create_dir(&next_folder).unwrap();

    fs::write(root.join(".gitignore"), ".next\n").unwrap();

    fs::write(subfolder.join("main.c"), "// Main file comment\nint x = 1;").unwrap();
    fs::write(next_folder.join("test.c"), "// Test file comment\nint y = 2;").unwrap();

    Command::new("git").current_dir(root).args(["init"]).output().unwrap();

    let uncomment_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("uncomment");

    let output = Command::new(&uncomment_path)
        .current_dir(&subfolder)
        .args([".", "--dry-run"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!("Command failed with stderr: {}", stderr);
    }

    assert!(
        stdout.contains("1 files processed"),
        "Expected to process only 1 file, but got: {}",
        stdout
    );
    assert!(
        stdout.contains("main.c"),
        "Expected to process main.c, but got: {}",
        stdout
    );
    assert!(
        !stdout.contains("test.c"),
        "Should not process test.c in .next folder, but got: {}",
        stdout
    );
}

#[test]
fn test_gitignore_with_no_gitignore_flag() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let subfolder = root.join("subfolder");
    let next_folder = subfolder.join(".next");

    fs::create_dir(&subfolder).unwrap();
    fs::create_dir(&next_folder).unwrap();

    fs::write(root.join(".gitignore"), ".next\n").unwrap();

    fs::write(subfolder.join("main.c"), "// Main file comment\nint x = 1;").unwrap();
    fs::write(next_folder.join("test.c"), "// Test file comment\nint y = 2;").unwrap();

    Command::new("git").current_dir(root).args(["init"]).output().unwrap();

    let uncomment_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("uncomment");

    let output = Command::new(&uncomment_path)
        .current_dir(&subfolder)
        .args([".", "--dry-run", "--no-gitignore"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("2 files processed"),
        "Expected to process 2 files with --no-gitignore, but got: {}",
        stdout
    );
}

#[test]
fn test_config_init_command() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let uncomment_path = get_binary_path();

    let output = Command::new(&uncomment_path)
        .current_dir(root)
        .args(["init", "--output", "test_config.toml"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Init command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let config_path = root.join("test_config.toml");
    assert!(config_path.exists(), "Config file was not created");

    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(config_content.contains("[global]"));
    assert!(config_content.contains("remove_todos = false"));
    assert!(config_content.contains("[languages.c]"));
    assert!(config_content.contains("[patterns."));
}

#[test]
fn test_config_basic_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create a simple config that removes TODO comments
    let config_content = r#"
[global]
remove_todos = true
remove_fixme = false
remove_docs = false
preserve_patterns = ["KEEP"]
"#;

    fs::write(root.join(".uncommentrc.toml"), config_content).unwrap();

    let test_file = root.join("test.c");
    let test_content = r#"// Header comment
// TODO: should be removed
// FIXME: should be preserved
// KEEP: should be preserved
// Regular comment
int hello(void) { return 0; }"#;

    fs::write(&test_file, test_content).unwrap();

    let uncomment_path = get_binary_path();

    let output = Command::new(&uncomment_path)
        .current_dir(root)
        .args(["test.c"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let result_content = fs::read_to_string(&test_file).unwrap();

    // TODO should be removed (due to config)
    assert!(!result_content.contains("TODO: should be removed"));

    // FIXME should be preserved (due to config)
    assert!(result_content.contains("FIXME: should be preserved"));

    assert!(result_content.contains("KEEP: should be preserved"));

    assert!(!result_content.contains("// Header comment"));
    assert!(!result_content.contains("// Regular comment"));
}

#[test]
fn test_nested_configuration() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create root config - preserves TODOs
    let root_config = r#"
[global]
remove_todos = false
remove_fixme = false
"#;
    fs::write(root.join(".uncommentrc.toml"), root_config).unwrap();

    let subdir = root.join("subdir");
    fs::create_dir(&subdir).unwrap();

    // Create subdirectory config - removes TODOs
    let sub_config = r#"
[global]
remove_todos = true
remove_fixme = false
"#;
    fs::write(subdir.join(".uncommentrc.toml"), sub_config).unwrap();

    let root_file = root.join("root_test.c");
    let sub_file = subdir.join("sub_test.c");

    let test_content = "// TODO: test comment\n// FIXME: test comment\nint hello(void) { return 0; }";
    fs::write(&root_file, test_content).unwrap();
    fs::write(&sub_file, test_content).unwrap();

    let uncomment_path = get_binary_path();

    let output = Command::new(&uncomment_path)
        .current_dir(root)
        .args(["root_test.c", "subdir/sub_test.c"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check root file - TODO should be preserved
    let root_result = fs::read_to_string(&root_file).unwrap();
    assert!(
        root_result.contains("TODO: test comment"),
        "Root file should preserve TODO"
    );
    assert!(
        root_result.contains("FIXME: test comment"),
        "Root file should preserve FIXME"
    );

    // Check sub file - TODO should be removed, FIXME preserved
    let sub_result = fs::read_to_string(&sub_file).unwrap();
    assert!(
        !sub_result.contains("TODO: test comment"),
        "Sub file should remove TODO"
    );
    assert!(
        sub_result.contains("FIXME: test comment"),
        "Sub file should preserve FIXME"
    );
}

#[test]
fn test_custom_config_file_path() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let custom_config = r#"
[global]
remove_todos = true
preserve_patterns = ["CUSTOM"]
"#;

    let custom_config_path = root.join("my_custom_config.toml");
    fs::write(&custom_config_path, custom_config).unwrap();

    let test_file = root.join("test.c");
    let test_content = "// TODO: should be removed\n// CUSTOM: should be preserved\nint hello(void) { return 0; }";
    fs::write(&test_file, test_content).unwrap();

    let uncomment_path = get_binary_path();

    let output = Command::new(&uncomment_path)
        .current_dir(root)
        .args(["--config", "my_custom_config.toml", "test.c"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let result = fs::read_to_string(&test_file).unwrap();
    assert!(!result.contains("TODO: should be removed"), "TODO should be removed");
    assert!(
        result.contains("CUSTOM: should be preserved"),
        "CUSTOM pattern should be preserved"
    );
}

#[test]
fn test_config_validation_errors() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let invalid_config = r#"
[global]
remove_todos = "invalid_boolean_value"  # Should fail TOML parsing
invalid_syntax_here
"#;

    fs::write(root.join(".uncommentrc.toml"), invalid_config).unwrap();

    fs::write(root.join("test.c"), "// comment\nint hello(void) { return 0; }").unwrap();

    let uncomment_path = get_binary_path();

    let output = Command::new(&uncomment_path)
        .current_dir(root)
        .args(["--config", ".uncommentrc.toml", "test.c"])
        .output()
        .unwrap();

    assert!(!output.status.success(), "Command should fail with invalid config");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to parse config")
            || stderr.contains("TOML parse error")
            || stderr.contains("invalid type"),
        "Should show validation error, got: {}",
        stderr
    );
}

#[test]
fn test_config_override_cli_options() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Create config that preserves TODOs
    let config_content = r#"
[global]
remove_todos = false
"#;

    fs::write(root.join(".uncommentrc.toml"), config_content).unwrap();

    let test_file = root.join("test.c");
    fs::write(&test_file, "// TODO: test comment\nint hello(void) { return 0; }").unwrap();

    let uncomment_path = get_binary_path();

    // Run with CLI flag that would remove TODOs (CLI should override config in future)
    let output = Command::new(&uncomment_path)
        .current_dir(root)
        .args(["--remove-todo", "test.c"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // CLI --remove-todo overrides config, so TODO should be removed
    let result = fs::read_to_string(&test_file).unwrap();
    assert!(
        !result.contains("// TODO: test comment"),
        "CLI --remove-todo should override config and remove TODO comments"
    );
    assert!(
        result.contains("int hello(void) { return 0; }"),
        "Code should be preserved"
    );
}

fn get_binary_path() -> std::path::PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("uncomment")
}
