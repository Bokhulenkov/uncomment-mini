#[derive(Debug, Clone)]
pub struct LanguageConfig {
    pub name: String,
    pub extensions: Vec<String>,
    pub comment_types: Vec<String>,
    pub doc_comment_types: Vec<String>,
    pub tslp_name: String,
}

impl LanguageConfig {
    pub fn new(
        name: &str,
        extensions: Vec<&str>,
        comment_types: Vec<&str>,
        doc_comment_types: Vec<&str>,
        tslp_name: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            extensions: extensions.iter().map(|&s| s.to_string()).collect(),
            comment_types: comment_types.iter().map(|&s| s.to_string()).collect(),
            doc_comment_types: doc_comment_types.iter().map(|&s| s.to_string()).collect(),
            tslp_name: tslp_name.to_string(),
        }
    }
}

impl LanguageConfig {
    pub fn c() -> Self {
        Self::new("c", vec!["c", "h"], vec!["comment"], vec!["comment"], "c")
    }

    pub fn cpp() -> Self {
        Self::new(
            "cpp",
            vec!["cpp", "cxx", "cc", "c++", "hpp", "hxx", "hh", "h++"],
            vec!["comment"],
            vec!["comment"],
            "cpp",
        )
    }

    pub fn objc() -> Self {
        Self::new("objc", vec!["m"], vec!["comment"], vec!["comment"], "objc")
    }

    pub fn swift() -> Self {
        Self::new(
            "swift",
            vec!["swift"],
            vec!["comment", "multiline_comment"],
            vec![],
            "swift",
        )
    }
}
