use ahash::AHashMap;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DetectionInfo {
    pub detected_languages: HashMap<String, usize>,
    pub configured_languages: usize,
    pub total_files: usize,
}

fn prompt_bool(prompt: &str, default: bool) -> Result<bool> {
    use std::io::{self, Write};

    print!("{} [{}]: ", prompt, if default { "Y/n" } else { "y/N" });
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(match input.as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => default,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub global: GlobalConfig,

    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,

    #[serde(default)]
    pub patterns: HashMap<String, PatternConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Whether to remove TODO comments
    #[serde(default = "default_false")]
    pub remove_todos: bool,

    /// Whether to remove FIXME comments
    #[serde(default = "default_false")]
    pub remove_fixme: bool,

    #[serde(default = "default_false")]
    pub remove_docs: bool,

    #[serde(default)]
    pub preserve_patterns: Vec<String>,

    #[serde(default = "default_true")]
    pub use_default_ignores: bool,

    #[serde(default = "default_true")]
    pub respect_gitignore: bool,

    #[serde(default = "default_false")]
    pub traverse_git_repos: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    #[serde(default)]
    pub preserve_patterns: Vec<String>,

    /// Override global remove_todos setting
    pub remove_todos: Option<bool>,

    /// Override global remove_fixme setting
    pub remove_fixme: Option<bool>,

    pub remove_docs: Option<bool>,

    pub use_default_ignores: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Whether to remove TODO comments
    pub remove_todos: Option<bool>,

    /// Whether to remove FIXME comments
    pub remove_fixme: Option<bool>,

    pub remove_docs: Option<bool>,

    #[serde(default)]
    pub preserve_patterns: Vec<String>,

    pub use_default_ignores: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub remove_todos: bool,
    pub remove_fixme: bool,
    pub remove_docs: bool,
    pub preserve_patterns: Vec<String>,
    pub use_default_ignores: bool,
    pub respect_gitignore: bool,
    pub traverse_git_repos: bool,
}

#[derive(Debug)]
pub struct ConfigManager {
    configs: Vec<(PathBuf, Config)>,

    path_configs: AHashMap<PathBuf, ResolvedConfig>,

    root_dir: PathBuf,
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            remove_todos: false,
            remove_fixme: false,
            remove_docs: false,
            preserve_patterns: Vec::new(),
            use_default_ignores: true,
            respect_gitignore: true,
            traverse_git_repos: false,
        }
    }
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.as_ref().display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.as_ref().display()))?;

        Ok(config)
    }

    pub fn template_clean() -> String {
        r#"[global]
remove_todos = false
remove_fixme = false
remove_docs = false
preserve_patterns = ["HACK", "WORKAROUND", "NOTE"]
use_default_ignores = true
respect_gitignore = true
traverse_git_repos = false

[languages.c]

[languages.cpp]

[languages.objc]
preserve_patterns = ["MARK:"]

[languages.swift]
preserve_patterns = ["MARK:", "swiftlint:"]

[patterns."tests/**/*"]
remove_todos = true

[patterns."**/*.generated.*"]
remove_docs = true
remove_todos = true
preserve_patterns = []
"#
        .to_string()
    }

    pub fn smart_template_with_info<P: AsRef<Path>>(project_dir: P) -> Result<(String, DetectionInfo)> {
        use crate::languages::registry::LanguageRegistry;
        use walkdir::WalkDir;

        let registry = LanguageRegistry::new();
        let mut detected_languages = HashMap::new();
        let mut total_files = 0;
        for entry in WalkDir::new(project_dir.as_ref())
            .max_depth(3)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if entry.file_type().is_file() {
                total_files += 1;
                if let Some(language) = registry.detect_language(entry.path()) {
                    *detected_languages.entry(language.name.clone()).or_insert(0) += 1;
                }
            }
        }

        if detected_languages.is_empty() {
            return Ok((
                Self::template_clean(),
                DetectionInfo {
                    detected_languages,
                    configured_languages: 0,
                    total_files,
                },
            ));
        }

        let mappings = Self::get_language_mappings();
        let mut config = String::from(
            r#"[global]
remove_todos = false
remove_fixme = false
remove_docs = false
preserve_patterns = ["HACK", "WORKAROUND", "NOTE"]
use_default_ignores = true
respect_gitignore = true
traverse_git_repos = false

"#,
        );
        let mut names: Vec<_> = detected_languages.keys().collect();
        names.sort();
        for name in names {
            if let Some(language_config) = mappings.get(name.as_str()) {
                config.push_str(language_config);
                config.push_str("\n\n");
            }
        }
        let info = DetectionInfo {
            configured_languages: detected_languages.len(),
            detected_languages,
            total_files,
        };
        Ok((config, info))
    }

    pub fn interactive_template_clean() -> Result<String> {
        use std::io::{self, Write};

        println!("🚀 Welcome to Uncomment Interactive Configuration!");
        println!("I'll help you create a customized configuration file.\n");

        let remove_todos = prompt_bool("Remove TODO comments by default? (y/n)", false)?;
        let remove_fixme = prompt_bool("Remove FIXME comments by default? (y/n)", false)?;
        let remove_docs = prompt_bool("Remove documentation comments by default? (y/n)", false)?;

        println!("\n📋 Available languages with grammar support:");
        let available_languages = vec![("c", "C"), ("cpp", "C++"), ("objc", "Objective-C"), ("swift", "Swift")];

        for (i, (name, desc)) in available_languages.iter().enumerate() {
            println!("  {}. {} - {}", i + 1, name, desc);
        }

        println!("\nSelect languages to include (comma-separated numbers, or 'all' for all, or 'skip' to skip):");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        let mut selected_languages = Vec::new();

        if input == "all" {
            selected_languages = available_languages.iter().map(|(name, _)| *name).collect();
        } else if input != "skip" {
            for num_str in input.split(',') {
                if let Ok(num) = num_str.trim().parse::<usize>()
                    && num > 0
                    && num <= available_languages.len()
                {
                    selected_languages.push(available_languages[num - 1].0);
                }
            }
        }

        let mut config = format!(
            r#"[global]
remove_todos = {remove_todos}
remove_fixme = {remove_fixme}
remove_docs = {remove_docs}
preserve_patterns = ["HACK", "WORKAROUND", "NOTE"]
use_default_ignores = true
respect_gitignore = true
traverse_git_repos = false

"#
        );

        let language_configs = Self::get_language_mappings();
        for lang in &selected_languages {
            if let Some(lang_config) = language_configs.get(*lang) {
                config.push_str(lang_config);
                config.push('\n');
            }
        }

        if !selected_languages.is_empty() {
            println!(
                "\n✅ Generated configuration with {} languages!",
                selected_languages.len()
            );
        }

        Ok(config)
    }

    fn get_language_mappings() -> std::collections::HashMap<&'static str, &'static str> {
        std::collections::HashMap::from([
            (
                "c",
                r#"[languages.c]
"#,
            ),
            (
                "cpp",
                r#"[languages.cpp]
"#,
            ),
            (
                "objc",
                r#"[languages.objc]
preserve_patterns = ["MARK:"]"#,
            ),
            (
                "swift",
                r#"[languages.swift]
preserve_patterns = ["MARK:", "swiftlint:"]"#,
            ),
        ])
    }

    pub fn merge_with(&self, other: &Config) -> Config {
        let mut merged = self.clone();

        merged.global.remove_todos = other.global.remove_todos;
        merged.global.remove_fixme = other.global.remove_fixme;
        merged.global.remove_docs = other.global.remove_docs;
        merged.global.use_default_ignores = other.global.use_default_ignores;
        merged.global.respect_gitignore = other.global.respect_gitignore;
        merged.global.traverse_git_repos = other.global.traverse_git_repos;

        let mut patterns = merged.global.preserve_patterns.clone();
        patterns.extend(other.global.preserve_patterns.iter().cloned());
        patterns.sort();
        patterns.dedup();
        merged.global.preserve_patterns = patterns;

        merged.languages.extend(
            other
                .languages
                .iter()
                .map(|(name, config)| (name.clone(), config.clone())),
        );
        merged.patterns.extend(
            other
                .patterns
                .iter()
                .map(|(pattern, config)| (pattern.clone(), config.clone())),
        );

        merged
    }
}

impl ConfigManager {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        let configs = Self::discover_configs(&root_dir)?;

        let mut manager = Self {
            configs,
            path_configs: AHashMap::new(),
            root_dir,
        };

        manager.precompute_configs()?;
        Ok(manager)
    }

    pub fn from_single_config<P: AsRef<Path>>(root_dir: P, config: Config) -> Result<Self> {
        let root_dir = root_dir.as_ref().to_path_buf();
        let configs = vec![(root_dir.clone(), config)];

        let mut manager = Self {
            configs,
            path_configs: AHashMap::new(),
            root_dir,
        };

        manager.precompute_configs()?;
        Ok(manager)
    }

    fn discover_configs(root_dir: &Path) -> Result<Vec<(PathBuf, Config)>> {
        let mut configs = Vec::new();

        for entry in walkdir::WalkDir::new(root_dir) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if matches!(file_name, ".uncommentrc.toml" | "uncomment.toml") {
                    match Config::from_file(path) {
                        Ok(config) => {
                            configs.push((path.to_path_buf(), config));
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to load config file {}: {e}", path.display());
                        }
                    }
                }
            }
        }

        if let Some(global_config_path) = Self::global_config_path()
            && global_config_path.exists()
        {
            match Config::from_file(&global_config_path) {
                Ok(config) => {
                    configs.push((global_config_path, config));
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load global config: {e}");
                }
            }
        }

        configs.sort_by_key(|(path, _)| path.components().count());

        Ok(configs)
    }

    fn global_config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("uncomment").join("config.toml"))
    }

    fn precompute_configs(&mut self) -> Result<()> {
        let mut dirs_to_process = vec![self.root_dir.clone()];

        for entry in walkdir::WalkDir::new(&self.root_dir) {
            let entry = entry?;
            if entry.path().is_dir() {
                dirs_to_process.push(entry.path().to_path_buf());
            }
        }

        for dir_path in dirs_to_process {
            let resolved = self.resolve_config_for_path(&dir_path);
            self.path_configs.insert(dir_path, resolved);
        }

        Ok(())
    }

    fn resolve_config_for_path(&self, path: &Path) -> ResolvedConfig {
        let mut base_config = Config::default();
        let global_config_path = Self::global_config_path();

        if let Some((_, global_config)) = self
            .configs
            .iter()
            .find(|(config_path, _)| global_config_path.as_ref() == Some(config_path))
        {
            base_config = base_config.merge_with(global_config);
        }

        let mut current_path = path;
        let mut applicable_configs = Vec::new();

        loop {
            for (config_path, config) in &self.configs {
                if let Some(config_dir) = config_path.parent()
                    && config_dir == current_path
                {
                    applicable_configs.push(config);
                }
            }

            if let Some(parent) = current_path.parent() {
                current_path = parent;
            } else {
                break;
            }
        }

        applicable_configs.reverse();
        for config in applicable_configs {
            base_config = base_config.merge_with(config);
        }

        ResolvedConfig {
            remove_todos: base_config.global.remove_todos,
            remove_fixme: base_config.global.remove_fixme,
            remove_docs: base_config.global.remove_docs,
            preserve_patterns: base_config.global.preserve_patterns,
            use_default_ignores: base_config.global.use_default_ignores,
            respect_gitignore: base_config.global.respect_gitignore,
            traverse_git_repos: base_config.global.traverse_git_repos,
        }
    }

    pub fn get_config_for_file<P: AsRef<Path>>(&self, file_path: P) -> ResolvedConfig {
        let file_path = file_path.as_ref();

        let absolute_file_path = if file_path.is_absolute() {
            file_path.to_path_buf()
        } else {
            std::env::current_dir().unwrap_or_default().join(file_path)
        };

        let dir_path = absolute_file_path.parent().unwrap_or(&absolute_file_path);

        self.path_configs
            .get(dir_path)
            .cloned()
            .unwrap_or_else(|| self.resolve_config_for_path(dir_path))
    }

    pub fn get_config_for_file_with_language<P: AsRef<Path>>(
        &self,
        file_path: P,
        language_name: &str,
    ) -> ResolvedConfig {
        let mut config = self.get_config_for_file(file_path);

        if let Some(lang_config) = self.get_language_config(language_name) {
            if let Some(remove_todos) = lang_config.remove_todos {
                config.remove_todos = remove_todos;
            }
            if let Some(remove_fixme) = lang_config.remove_fixme {
                config.remove_fixme = remove_fixme;
            }
            if let Some(remove_docs) = lang_config.remove_docs {
                config.remove_docs = remove_docs;
            }
            if let Some(use_default_ignores) = lang_config.use_default_ignores {
                config.use_default_ignores = use_default_ignores;
            }

            config
                .preserve_patterns
                .extend(lang_config.preserve_patterns.iter().cloned());
            config.preserve_patterns.sort();
            config.preserve_patterns.dedup();
        }

        config
    }

    pub fn get_language_config(&self, language_name: &str) -> Option<LanguageConfig> {
        for (_, config) in self.configs.iter().rev() {
            if let Some(lang_config) = config.languages.get(language_name) {
                return Some(lang_config.clone());
            }

            if let Some((_, lang_config)) = config
                .languages
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(language_name))
            {
                return Some(lang_config.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_template() {
        let template = Config::template_clean();
        assert!(template.contains("[global]"));
        assert!(template.contains("[languages.c]"));
        assert!(template.contains("[patterns."));
    }

    #[test]
    fn test_config_merging() {
        let base = Config {
            global: GlobalConfig {
                remove_todos: false,
                preserve_patterns: vec!["TODO".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let override_config = Config {
            global: GlobalConfig {
                remove_todos: true,
                preserve_patterns: vec!["FIXME".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };

        let merged = base.merge_with(&override_config);
        assert!(merged.global.remove_todos);
        assert_eq!(merged.global.preserve_patterns, vec!["FIXME", "TODO"]);
    }
}
