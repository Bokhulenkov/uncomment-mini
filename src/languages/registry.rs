use crate::languages::config::LanguageConfig;
use ahash::AHashMap;
use std::path::Path;
use std::sync::Arc;

pub struct LanguageRegistry {
    languages: AHashMap<String, Arc<LanguageConfig>>,
    extension_map: AHashMap<String, String>,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            languages: AHashMap::new(),
            extension_map: AHashMap::new(),
        };

        registry.register_default_languages();
        registry
    }

    fn register_default_languages(&mut self) {
        let configs = vec![
            LanguageConfig::c(),
            LanguageConfig::cpp(),
            LanguageConfig::objc(),
            LanguageConfig::swift(),
        ];

        for config in configs {
            self.register_language(config);
        }
    }

    fn register_language(&mut self, config: LanguageConfig) {
        let config = Arc::new(config);
        let name_lower = config.name.to_lowercase();

        for extension in &config.extensions {
            let normalized_ext = extension.trim_start_matches('.').to_lowercase();
            self.extension_map.insert(normalized_ext, name_lower.clone());
        }

        self.languages.insert(name_lower, config);
    }

    pub fn get_language(&self, name: &str) -> Option<&LanguageConfig> {
        self.languages.get(&name.to_lowercase()).map(Arc::as_ref)
    }

    pub fn detect_language(&self, file_path: &Path) -> Option<&LanguageConfig> {
        let language_name = self.detect_language_name(file_path)?;
        self.languages.get(language_name).map(Arc::as_ref)
    }

    #[must_use]
    pub fn detect_language_arc(&self, file_path: &Path) -> Option<Arc<LanguageConfig>> {
        let language_name = self.detect_language_name(file_path)?;
        self.languages.get(language_name).cloned()
    }

    fn detect_language_name(&self, file_path: &Path) -> Option<&str> {
        let extension = file_path.extension()?.to_str()?.to_lowercase();
        self.extension_map.get(&extension).map(String::as_str)
    }

    pub fn get_supported_extensions(&self) -> Vec<String> {
        self.extension_map.keys().cloned().collect()
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_registry_creation() {
        let registry = LanguageRegistry::new();
        assert_eq!(registry.languages.len(), 4);
        assert!(!registry.extension_map.is_empty());
    }

    #[test]
    fn test_language_detection_by_path() {
        let registry = LanguageRegistry::new();
        for (path, expected) in [
            ("test.c", "c"),
            ("test.cpp", "cpp"),
            ("test.m", "objc"),
            ("test.swift", "swift"),
        ] {
            assert_eq!(registry.detect_language(&PathBuf::from(path)).unwrap().name, expected);
        }
        assert!(registry.detect_language(&PathBuf::from("file.unknown")).is_none());
    }

    #[test]
    fn test_supported_languages() {
        let registry = LanguageRegistry::new();
        for name in ["c", "cpp", "objc", "swift"] {
            assert!(registry.get_language(name).is_some());
        }
        assert!(registry.get_language("python").is_none());
    }

    #[test]
    fn test_supported_extensions() {
        let registry = LanguageRegistry::new();
        let extensions = registry.get_supported_extensions();
        for extension in [
            "c", "h", "cpp", "cxx", "cc", "c++", "hpp", "hxx", "hh", "h++", "m", "swift",
        ] {
            assert!(extensions.contains(&extension.to_string()));
        }
        assert!(!extensions.contains(&"mm".to_string()));
    }

    #[test]
    fn test_case_insensitive_extension_detection() {
        let registry = LanguageRegistry::new();
        assert_eq!(
            registry.detect_language(&PathBuf::from("test.CPP")).unwrap().name,
            "cpp"
        );
        assert_eq!(
            registry.detect_language(&PathBuf::from("test.SWIFT")).unwrap().name,
            "swift"
        );
    }
}
