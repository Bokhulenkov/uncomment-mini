use tree_sitter::Node;

pub trait LanguageHandler {
    fn is_documentation_comment(&self, node: &Node, parent: Option<Node>, source: &str) -> Option<bool>;

    fn should_preserve_comment(&self, node: &Node, parent: Option<Node>, source: &str) -> Option<bool>;
}

pub struct DefaultHandler;

impl LanguageHandler for DefaultHandler {
    fn is_documentation_comment(&self, _node: &Node, _parent: Option<Node>, _source: &str) -> Option<bool> {
        None
    }

    fn should_preserve_comment(&self, _node: &Node, _parent: Option<Node>, _source: &str) -> Option<bool> {
        None
    }
}

pub fn get_handler(language_name: &str) -> Box<dyn LanguageHandler> {
    match language_name.to_lowercase().as_str() {
        "c" | "cpp" | "objc" => Box::new(CFamilyHandler),
        _ => Box::new(DefaultHandler),
    }
}

pub struct CFamilyHandler;

impl LanguageHandler for CFamilyHandler {
    fn is_documentation_comment(&self, _node: &Node, _parent: Option<Node>, _source: &str) -> Option<bool> {
        None
    }

    fn should_preserve_comment(&self, node: &Node, _parent: Option<Node>, source: &str) -> Option<bool> {
        if node.kind() != "comment" {
            return None;
        }

        if self.is_trailing_preprocessor_comment(node, source) {
            return Some(true);
        }

        None
    }
}

impl CFamilyHandler {
    fn is_trailing_preprocessor_comment(&self, node: &Node, source: &str) -> bool {
        let start = node.start_byte();
        if start > source.len() {
            return false;
        }

        let line_start = match memchr::memrchr(b'\n', &source.as_bytes()[..start]) {
            Some(pos) => pos + 1,
            None => 0,
        };

        let before = &source[line_start..start];
        before.trim_start().starts_with('#')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_handler() {
        let _handler = DefaultHandler;
    }

    #[test]
    fn test_handler_factory() {
        let _c_handler = get_handler("c");
        let _cpp_handler = get_handler("cpp");
        let _objc_handler = get_handler("objc");
        let _swift_handler = get_handler("swift");
    }
}
