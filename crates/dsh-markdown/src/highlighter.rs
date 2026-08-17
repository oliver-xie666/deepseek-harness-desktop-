use serde::{Deserialize, Serialize};
use tree_sitter::{Language, Parser};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenType {
    Keyword,
    Function,
    Type,
    String,
    Number,
    Comment,
    Operator,
    Variable,
    Punctuation,
    Default,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HighlightSpan {
    pub text: String,
    pub token_type: TokenType,
}

pub struct CodeHighlighter;

impl CodeHighlighter {
    pub fn get_language(lang: &str) -> Option<Language> {
        match lang.to_lowercase().as_str() {
            "rust" | "rs" => Some(tree_sitter_rust::LANGUAGE.into()),
            "typescript" | "ts" | "javascript" | "js" => {
                Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            }
            "python" | "py" => Some(tree_sitter_python::LANGUAGE.into()),
            "json" => Some(tree_sitter_json::LANGUAGE.into()),
            _ => None,
        }
    }

    /// Highlights code into a sequence of tokens
    pub fn highlight(code: &str, lang: &str) -> Vec<HighlightSpan> {
        let language = match Self::get_language(lang) {
            Some(l) => l,
            None => {
                return vec![HighlightSpan {
                    text: code.to_string(),
                    token_type: TokenType::Default,
                }]
            }
        };

        let mut parser = Parser::new();
        if parser.set_language(&language).is_err() {
            return vec![HighlightSpan {
                text: code.to_string(),
                token_type: TokenType::Default,
            }];
        }

        let mut spans = Vec::new();

        // Line-based fallback syntax tokenization with keyword detection
        for line in code.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") || trimmed.starts_with("#") {
                spans.push(HighlightSpan {
                    text: line.to_string() + "\n",
                    token_type: TokenType::Comment,
                });
            } else if trimmed.starts_with("pub ")
                || trimmed.starts_with("fn ")
                || trimmed.starts_with("def ")
                || trimmed.starts_with("import ")
                || trimmed.starts_with("let ")
                || trimmed.starts_with("const ")
            {
                spans.push(HighlightSpan {
                    text: line.to_string() + "\n",
                    token_type: TokenType::Keyword,
                });
            } else {
                spans.push(HighlightSpan {
                    text: line.to_string() + "\n",
                    token_type: TokenType::Default,
                });
            }
        }

        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_rust() {
        let code = "fn main() {\n    // comment\n    println!(\"hello\");\n}";
        let spans = CodeHighlighter::highlight(code, "rust");
        assert!(!spans.is_empty());
    }
}
