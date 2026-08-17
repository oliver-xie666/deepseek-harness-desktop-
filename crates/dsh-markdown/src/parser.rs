use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InlineSpan {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Link { text: String, url: String },
    FilePath { path: String, line: Option<usize> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AlertType {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MarkdownBlock {
    Heading {
        level: u8,
        inlines: Vec<InlineSpan>,
    },
    Paragraph {
        inlines: Vec<InlineSpan>,
    },
    CodeBlock {
        language: String,
        code: String,
    },
    Blockquote {
        inlines: Vec<InlineSpan>,
    },
    Alert {
        alert_type: AlertType,
        inlines: Vec<InlineSpan>,
    },
    List {
        is_ordered: bool,
        items: Vec<Vec<InlineSpan>>,
    },
    HorizontalRule,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MarkdownDocument {
    pub blocks: Vec<MarkdownBlock>,
}

pub struct StreamingMarkdownParser {
    buffer: String,
    finalized_blocks: Vec<MarkdownBlock>,
    pub tail_block: Option<MarkdownBlock>,
}

impl Default for StreamingMarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingMarkdownParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            finalized_blocks: Vec::new(),
            tail_block: None,
        }
    }

    /// Appends new token text and updates parsed blocks
    pub fn append_chunk(&mut self, chunk: &str) {
        self.buffer.push_str(chunk);
        self.reparse();
    }

    /// Full reparse of current buffer
    pub fn reparse(&mut self) {
        self.finalized_blocks = Self::parse_markdown(&self.buffer);
    }

    pub fn get_document(&self) -> MarkdownDocument {
        MarkdownDocument {
            blocks: self.finalized_blocks.clone(),
        }
    }

    pub fn parse_markdown(text: &str) -> Vec<MarkdownBlock> {
        let parser = Parser::new(text);
        let mut blocks = Vec::new();
        let mut current_inlines: Vec<InlineSpan> = Vec::new();
        let mut current_code = String::new();
        let mut current_lang = String::new();
        let mut in_code_block = false;
        let mut current_heading_level = 1;
        let mut is_bold = false;
        let mut is_italic = false;
        let mut current_link_url = String::new();
        let mut in_link = false;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    current_heading_level = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    current_inlines.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    blocks.push(MarkdownBlock::Heading {
                        level: current_heading_level,
                        inlines: std::mem::take(&mut current_inlines),
                    });
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    current_lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    current_code.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    in_code_block = false;
                    blocks.push(MarkdownBlock::CodeBlock {
                        language: current_lang.clone(),
                        code: current_code.clone(),
                    });
                }
                Event::Start(Tag::Paragraph) => {
                    current_inlines.clear();
                }
                Event::End(TagEnd::Paragraph) => {
                    if !current_inlines.is_empty() {
                        // Check if it's an alert
                        let is_alert = current_inlines.first().and_then(|span| match span {
                            InlineSpan::Text(t) => {
                                if t.starts_with("[!NOTE]") {
                                    Some(AlertType::Note)
                                } else if t.starts_with("[!TIP]") {
                                    Some(AlertType::Tip)
                                } else if t.starts_with("[!WARNING]") {
                                    Some(AlertType::Warning)
                                } else if t.starts_with("[!IMPORTANT]") {
                                    Some(AlertType::Important)
                                } else if t.starts_with("[!CAUTION]") {
                                    Some(AlertType::Caution)
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        });

                        if let Some(alert_type) = is_alert {
                            blocks.push(MarkdownBlock::Alert {
                                alert_type,
                                inlines: std::mem::take(&mut current_inlines),
                            });
                        } else {
                            blocks.push(MarkdownBlock::Paragraph {
                                inlines: std::mem::take(&mut current_inlines),
                            });
                        }
                    }
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    current_inlines.clear();
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if !current_inlines.is_empty() {
                        blocks.push(MarkdownBlock::Blockquote {
                            inlines: std::mem::take(&mut current_inlines),
                        });
                    }
                }
                Event::Start(Tag::Strong) => is_bold = true,
                Event::End(TagEnd::Strong) => is_bold = false,
                Event::Start(Tag::Emphasis) => is_italic = true,
                Event::End(TagEnd::Emphasis) => is_italic = false,
                Event::Start(Tag::Link { dest_url, .. }) => {
                    in_link = true;
                    current_link_url = dest_url.to_string();
                }
                Event::End(TagEnd::Link) => {
                    in_link = false;
                }
                Event::Code(code) => {
                    current_inlines.push(InlineSpan::Code(code.to_string()));
                }
                Event::Text(text) => {
                    if in_code_block {
                        current_code.push_str(&text);
                    } else if in_link {
                        current_inlines.push(InlineSpan::Link {
                            text: text.to_string(),
                            url: current_link_url.clone(),
                        });
                    } else if is_bold {
                        current_inlines.push(InlineSpan::Bold(text.to_string()));
                    } else if is_italic {
                        current_inlines.push(InlineSpan::Italic(text.to_string()));
                    } else {
                        current_inlines.push(InlineSpan::Text(text.to_string()));
                    }
                }
                Event::Rule => {
                    blocks.push(MarkdownBlock::HorizontalRule);
                }
                _ => {}
            }
        }

        blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_basic() {
        let md = "# Hello World\n\nThis is a **bold** paragraph with `inline code`.\n\n```rust\nfn main() {}\n```";
        let doc = StreamingMarkdownParser::parse_markdown(md);

        assert_eq!(doc.len(), 3);
        match &doc[0] {
            MarkdownBlock::Heading { level, .. } => assert_eq!(*level, 1),
            _ => panic!("Expected Heading"),
        }
        match &doc[1] {
            MarkdownBlock::Paragraph { inlines } => {
                assert!(inlines.contains(&InlineSpan::Bold("bold".to_string())));
                assert!(inlines.contains(&InlineSpan::Code("inline code".to_string())));
            }
            _ => panic!("Expected Paragraph"),
        }
        match &doc[2] {
            MarkdownBlock::CodeBlock { language, code } => {
                assert_eq!(language, "rust");
                assert_eq!(code, "fn main() {}\n");
            }
            _ => panic!("Expected CodeBlock"),
        }
    }
}
