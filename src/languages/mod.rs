use std::path::Path;
use tree_sitter::Language;

#[derive(Clone, Copy)]
pub enum SourceLanguage {
    C,
    ObjectiveC,
    Cpp,
    Swift,
}

impl SourceLanguage {
    pub fn for_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "h" => Some(Self::C),
            "m" => Some(Self::ObjectiveC),
            "mm" => Some(Self::Cpp),
            "swift" => Some(Self::Swift),
            _ => None,
        }
    }

    pub fn grammar(self) -> Language {
        match self {
            Self::C => tree_sitter_c::LANGUAGE.into(),
            Self::ObjectiveC => tree_sitter_objc::LANGUAGE.into(),
            Self::Cpp => tree_sitter_cpp::LANGUAGE.into(),
            Self::Swift => tree_sitter_swift::LANGUAGE.into(),
        }
    }
}
