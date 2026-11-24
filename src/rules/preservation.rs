// Единственный список шаблонов сохраняемых комментариев. После изменения пересоберите бинарник.
pub const PRESERVE_PATTERNS: &[&str] = &["HACK", "WORKAROUND", "~keep"];

pub fn is_documentation(content: &str) -> bool {
    let content = content.trim_start();
    ["/**", "///", "//!"].iter().any(|prefix| content.starts_with(prefix))
}

pub fn should_preserve(content: &str, before_comment: &str, is_swift: bool) -> bool {
    is_documentation(content)
        || PRESERVE_PATTERNS.iter().any(|pattern| content.contains(pattern))
        || (!is_swift && before_comment.trim_start().starts_with('#'))
}
