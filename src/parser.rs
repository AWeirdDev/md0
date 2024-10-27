use lazy_static::lazy_static;
use pyo3::prelude::*;
use regex::Regex;

macro_rules! make_regex {
    ($id:ident, $re:literal) => {
        lazy_static! {
            static ref $id: Regex = {
                let re = Regex::new($re);

                match re {
                    Ok(re) => re,
                    Err(err) => panic!("{:#?}", err),
                }
            };
        }
    };
}

make_regex!(HEADING_RE, r"(?m)^(#{1,6})\s+(.+)$");
make_regex!(FENCE_RE, r"(?m)^```([0-9a-zA-Z+-_]*)\s*$");
make_regex!(LINK_RE, r"(?m)\[([^\]]+)\]\(([^\)]+)\)");
make_regex!(IMAGE_RE, r"(?m)\!\[([^\]]+)\]\(([^\)]+)\)");

#[pyclass]
#[derive(Clone)]
pub(crate) enum Token {
    Heading { level: u8, content: String },
    Paragraph(String, Vec<Metadata>),
    HorizontalRule(),
    Code { language: String, content: String },
}

#[pymethods]
impl Token {
    #[pyo3(name = "__repr__")]
    pub fn py_repr(&self) -> String {
        match self {
            Self::Heading { level, content } => format!("Heading({level}, {content:?})"),
            Self::Paragraph(content, meta) => format!("Paragraph({content:?}, {meta:?})"),
            Self::HorizontalRule() => "HorizontalRule".to_string(),
            Self::Code { language, content } => format!("Code({language:?}, {content:?})"),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub(crate) enum Metadata {
    Link {
        location: (usize, usize),
        label: String,
        url: String,
    },
    Image {
        location: (usize, usize),
        label: String,
        url: String,
    },
}

#[pymethods]
impl Metadata {
    #[pyo3(name = "__repr__")]
    pub fn py_repr(&self) -> String {
        match self {
            Self::Link {
                location,
                label,
                url,
            } => format!("Link({location:?}, {label:?}, {url:?})"),
            Self::Image {
                location,
                label,
                url,
            } => format!("Image({location:?}, {label:?}, {url:?})"),
        }
    }
}

impl std::fmt::Debug for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.py_repr())
    }
}

impl Metadata {
    /// Parses and returns the link metadata(s), if any.
    pub(crate) fn links(paragraph: &String) -> Vec<Self> {
        let lre = LINK_RE.captures_iter(paragraph);

        lre.map(|c| {
            // Links
            let (start, end) = {
                let range = c.get(0).unwrap().range();
                (range.start, range.end)
            };

            Metadata::Link {
                location: (start, end),
                label: c[1].to_string(),
                url: c[2].to_string(),
            }
        })
        .collect::<Vec<_>>()
    }

    /// Parses and returns the image metadata(s), if any.
    pub(crate) fn images(paragraph: &String) -> Vec<Self> {
        let ire = IMAGE_RE.captures_iter(paragraph);

        ire.map(|c| {
            // Links
            let (start, end) = {
                let range = c.get(0).unwrap().range();
                (range.start, range.end)
            };

            Metadata::Image {
                location: (start, end),
                label: c[1].to_string(),
                url: c[2].to_string(),
            }
        })
        .collect::<Vec<_>>()
    }
}

pub(crate) type Tokens = Vec<Token>;

/// Parses a Markdown string into a series of tokens.
///
/// # Example
///
/// ```rust
/// let text = "[Google](https://google.com)".to_string();
/// let metadata = Metadata::link(&text);
///
/// assert!(matches!(metadata, Some(Metadata::Link { .. })));
/// ```
/// # Returns
///
/// A vector of tokens
pub(crate) fn parse(input: String) -> PyResult<Tokens> {
    let lines = input.split('\n').map(|s| s.to_string()).collect::<Vec<_>>();

    let mut tokens: Tokens = Vec::new();
    let mut i = 0_usize;

    'consumer: while i < lines.len() {
        let line = &lines[i];

        if line.is_empty() {
            i += 1;
            continue 'consumer;
        }

        let hre = HEADING_RE.captures(&line);

        // Heading
        if let Some(c) = hre {
            tokens.push(Token::Heading {
                level: c[1].to_string().len() as u8,
                content: c[2].to_string(),
            });
        } else {
            let mut contents: Vec<String> = vec![];

            'collector: while i < lines.len() {
                let line = &lines[i];

                if line.trim().is_empty() {
                    break 'collector;
                }

                // "---" handling (horizontal rule or heading)
                if line.starts_with("---") && line.trim_matches('-').is_empty() {
                    if contents.is_empty() {
                        tokens.push(Token::HorizontalRule());
                    } else {
                        // If we have something like:
                        // ```markdown
                        // Only one new line!
                        // Hello, guys!
                        // ---
                        // ```
                        // We should ONLY collect "Hello, guys!"
                        // The `contents`:
                        // ["Only one new line!", "Hello, guys!"]
                        // So we should be getting [-1] as the heading, [:-1] as the content (before)

                        let before = &contents[..contents.len() - 1].join(" ");
                        let heading = &contents[contents.len() - 1];

                        if !before.is_empty() {
                            tokens.push(Token::Paragraph(before.to_string(), vec![]));
                        }
                        tokens.push(Token::Heading {
                            level: 1,
                            content: heading.to_string(),
                        });
                    }

                    i += 1;
                    continue 'consumer;
                }

                let fre = FENCE_RE.captures(&line);

                if let Some(c) = fre {
                    tokens.push(Token::Paragraph(contents.join(" "), vec![]));
                    contents.clear();

                    let language = c[1].to_string();
                    let mut code = String::new();

                    i += 1;
                    'code_collector: while i < lines.len() {
                        let line = &lines[i];

                        if line.trim() == "```" {
                            break 'code_collector;
                        }

                        code += line;
                        code += "\n";

                        i += 1;
                    }

                    tokens.push(Token::Code {
                        language,
                        content: code,
                    });

                    i += 1;
                    continue 'consumer;
                }

                contents.push(line.to_owned());
                i += 1;
            }

            let paragraph = contents.join(" ");
            let mut metadatas: Vec<Metadata> = vec![];

            // Process links
            for item in Metadata::links(&paragraph) {
                metadatas.push(item);
            }

            // Process images
            for item in Metadata::images(&paragraph) {
                metadatas.push(item);
            }

            tokens.push(Token::Paragraph(paragraph, metadatas));
            continue;
        }

        i += 1;
    }

    Ok(tokens)
}

pub(crate) fn tokens_to_html(tokens: Tokens) -> PyResult<String> {
    let mut contents: Vec<String> = vec![];

    for item in tokens {
        match item {
            Token::Paragraph(s, ..) => {
                contents.push(format!("<p>{}</p>", html_escape::encode_text(&s)))
            }
            Token::Code { content, .. } => contents.push(format!(
                "<pre><code>{}</code></pre>",
                html_escape::encode_text::<String>(&content).to_string()
            )),
            Token::Heading { level, content } => contents.push(format!(
                "<h{}>{}</h{}>",
                level,
                html_escape::encode_text(&content),
                level
            )),
            Token::HorizontalRule() => contents.push("<hr />".to_string()),
        }
    }

    Ok(contents.join("\n"))
}
