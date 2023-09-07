use lazy_static::lazy_static;
use log::info;
use mdbook::book::Book;
use mdbook::errors::Error;
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook::BookItem;

use regex::{Captures, Regex};
pub struct Private;

const STYLE_CONTENT: &str = "position: relative; padding: 20px 20px;";
const STYLE_NOTICE: &str = "position: absolute; top: 0; right: 5px; font-size: 80%; opacity: 0.4;";

impl Private {
    pub fn new() -> Private {
        Private
    }
}

impl Default for Private {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor for Private {
    fn name(&self) -> &str {
        "private"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book, Error> {
        info!("Running mdbook-private preprocessor");

        // Handle preprocessor configuration
        let mut remove = false;
        let mut style = true;
        let mut notice = "CONFIDENTIAL";
        if let Some(private_cfg) = ctx.config.get_preprocessor(self.name()) {
            if private_cfg.contains_key("remove") {
                let cfg_remove = private_cfg.get("remove").unwrap();
                remove = cfg_remove.as_bool().unwrap();
            }
            if private_cfg.contains_key("style") {
                let cfg_style = private_cfg.get("style").unwrap();
                style = cfg_style.as_bool().unwrap();

                if private_cfg.contains_key("notice") {
                    let cfg_notice = private_cfg.get("notice").unwrap();
                    notice = cfg_notice.as_str().unwrap();
                }
            }
        }

        lazy_static! {
            static ref RE: Regex = Regex::new(r"<!--\s*private\b\s*[\r?\n]?((?s).*?)[\r?\n]?\s*-->").unwrap();
        }

        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                info!("Processing chapter '{}'", &chapter.name);
                let result = if remove {
                    RE.replace_all(chapter.content.as_str(), "")
                } else {
                    RE.replace_all(chapter.content.as_str(), |caps: &Captures| {
                        if style {
                            format!(
                                "<blockquote style='{}'><span style='{}'>{}</span>{}</blockquote>",
                                &STYLE_CONTENT, STYLE_NOTICE, &notice, &caps[1]
                            )
                        } else {
                            caps[1].to_string()
                        }
                    })
                };

                chapter.content = result.to_string();
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn private_remove_preprocessor_run() {
        let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {
                                "remove": true
                            }
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n<!--private\nHello world!\n\nSome more text\n123!@#\n-->\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let output_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {
                                "remove": true
                            }
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let input_json = input_json.as_bytes();
        let output_json = output_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let (_, expected_book) =
            mdbook::preprocess::CmdPreprocessor::parse_input(output_json).unwrap();

        let result = Private::new().run(&ctx, book);
        assert!(result.is_ok());

        let actual_book = result.unwrap();
        assert_eq!(actual_book, expected_book);
    }

    #[test]
    fn private_keep_preprocessor_run() {
        let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n<!--private\nHello world!\n\nSome more text\n123!@#\n-->\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let output_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n<blockquote style='position: relative; padding: 20px 20px;'><span style='position: absolute; top: 0; right: 5px; font-size: 80%; opacity: 0.4;'>CONFIDENTIAL</span>Hello world!\n\nSome more text\n123!@#</blockquote>\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let input_json = input_json.as_bytes();
        let output_json = output_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let (_, expected_book) =
            mdbook::preprocess::CmdPreprocessor::parse_input(output_json).unwrap();

        let result = Private::new().run(&ctx, book);
        assert!(result.is_ok());

        let actual_book = result.unwrap();
        assert_eq!(actual_book, expected_book);
    }

    #[test]
    fn private_remove_robustly_run() {
        let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {
                                "remove": true
                            }
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n<!--private Hello world! -->\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let output_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {
                                "remove": true
                            }
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let input_json = input_json.as_bytes();
        let output_json = output_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let (_, expected_book) =
            mdbook::preprocess::CmdPreprocessor::parse_input(output_json).unwrap();

        let result = Private::new().run(&ctx, book);
        assert!(result.is_ok());

        let actual_book = result.unwrap();
        assert_eq!(actual_book, expected_book);
    }

    #[test]
    fn private_keep_robustly_run() {
        let input_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n<!--private Hello world! -->\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let output_json = r##"[
                {
                    "root": "/path/to/book",
                    "config": {
                        "book": {
                            "authors": ["AUTHOR"],
                            "language": "en",
                            "multilingual": false,
                            "src": "src",
                            "title": "TITLE"
                        },
                        "preprocessor": {
                            "private": {}
                        }
                    },
                    "renderer": "html",
                    "mdbook_version": "0.4.21"
                },
                {
                    "sections": [
                        {
                            "Chapter": {
                                "name": "Chapter 1",
                                "content": "# Chapter 1\n<blockquote style='position: relative; padding: 20px 20px;'><span style='position: absolute; top: 0; right: 5px; font-size: 80%; opacity: 0.4;'>CONFIDENTIAL</span>Hello world!</blockquote>\nThe End",
                                "number": [1],
                                "sub_items": [],
                                "path": "chapter_1.md",
                                "source_path": "chapter_1.md",
                                "parent_names": []
                            }
                        }
                    ],
                    "__non_exhaustive": null
                }
            ]"##;
        let input_json = input_json.as_bytes();
        let output_json = output_json.as_bytes();

        let (ctx, book) = mdbook::preprocess::CmdPreprocessor::parse_input(input_json).unwrap();
        let (_, expected_book) =
            mdbook::preprocess::CmdPreprocessor::parse_input(output_json).unwrap();

        let result = Private::new().run(&ctx, book);
        assert!(result.is_ok());

        let actual_book = result.unwrap();
        assert_eq!(actual_book, expected_book);
    }

}
