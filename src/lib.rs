use std::sync::LazyLock;

use log::info;
use mdbook::book::Book;
use mdbook::book::SectionNumber;
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
        let mut prefix = "_";
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
            if private_cfg.contains_key("chapter-prefix") {
                let cfg_prefix = private_cfg.get("chapter-prefix").unwrap();
                prefix = cfg_prefix.as_str().unwrap();
            }
        }

        static RE: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r"<!--\s*private\b\s*[\r?\n]?((?s).*?)[\r?\n]?\s*-->[\r?\n]?").unwrap()
        });

        // Handle private content blocks
        book.for_each_mut(|item: &mut BookItem| {
            if let BookItem::Chapter(ref mut chapter) = *item {
                info!("Processing chapter '{}'", &chapter.name);
                let result = if remove {
                    RE.replace_all(chapter.content.as_str(), "")
                } else {
                    RE.replace_all(chapter.content.as_str(), |caps: &Captures| {
                        if style {
                            format!(
                                "<blockquote style='{}'><span style='{}'>{}</span>{}</blockquote>\n",
                                &STYLE_CONTENT, STYLE_NOTICE, &notice, &caps[1]
                            )
                        } else {
                            caps[1].to_string() + "\n"
                        }
                    })
                };

                chapter.content = result.to_string();
            }
        });

        // Handle private chapters
        if remove {
            let mut private_book = Book::new();
            book.sections
                .iter()
                .filter_map(|section| process_item(section.clone(), prefix))
                .for_each(|item| {
                    private_book.push_item(item);
                });

            update_section_numbers(&mut private_book);

            return Ok(private_book);
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

/// Align section numbers with visible sections
fn update_section_numbers(book: &mut Book) {
    let mut current_number: Vec<u32> = Vec::new();

    fn update_chapter_numbers(chapters: &mut [BookItem], current_number: &mut Vec<u32>) {
        let mut section_counter = 1;

        for item in chapters.iter_mut() {
            if let BookItem::Chapter(ref mut chapter) = item {
                if chapter.number.is_some() {
                    // Only renumber numbered chapters
                    current_number.push(section_counter);
                    chapter.number = Some(SectionNumber(current_number.clone()));
                    update_chapter_numbers(&mut chapter.sub_items, current_number);
                    current_number.pop();
                    section_counter += 1;
                }
            }
        }
    }

    update_chapter_numbers(&mut book.sections, &mut current_number);
}

fn process_item(item: BookItem, prefix: &str) -> Option<BookItem> {
    match item {
        BookItem::Chapter(ch) => {
            if ch
                .source_path
                .as_ref()?
                .file_name()?
                .to_str()?
                .starts_with(prefix)
            {
                info!("Deleting chapter {}", ch.source_path.as_ref()?.display());
                return None;
            }

            let mut private_ch = ch.clone();
            private_ch.sub_items.clear();

            for sub in &ch.sub_items {
                if let Some(processed_sub) = process_item(sub.clone(), prefix) {
                    private_ch.sub_items.push(processed_sub);
                }
            }

            Some(BookItem::Chapter(private_ch))
        }
        _ => Some(item),
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
                                "content": "# Chapter 1\nThe End",
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
                                "content": "# Chapter 1\nThe End",
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

    #[test]
    fn private_keep_chapters_run() {
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
                "mdbook_version": "0.4.32"
              },
              {
                "sections": [
                  {
                    "Chapter": {
                      "name": "Chapter 1",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n<!--private\nThis is some highly confidential material which we want to remove when sharing with external parties.\n\nAnother *line*.\n\n# A title that should remain a title  \nYet another **line**.\n-->\n",
                      "number": [1],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis chapter will be removed if private is enabled\n",
                            "number": [1, 1],
                            "sub_items": [],
                            "path": "_chapter_1_sub.md",
                            "source_path": "_chapter_1_sub.md",
                            "parent_names": ["Chapter 1"]
                          }
                        }
                      ],
                      "path": "chapter_1.md",
                      "source_path": "chapter_1.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 2",
                      "content": "# Chapter 2\n\nThis chapter and it's subchapters will be removed if private is enabled\n",
                      "number": [2],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis will be removed if private is enabled because it's parent chapter is set to be removed.\n",
                            "number": [2, 1],
                            "sub_items": [],
                            "path": "chapter_2_sub.md",
                            "source_path": "chapter_2_sub.md",
                            "parent_names": ["Chapter 2"]
                          }
                        }
                      ],
                      "path": "_chapter_2.md",
                      "source_path": "_chapter_2.md",
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
                "mdbook_version": "0.4.32"
              },
              {
                "sections": [
                  {
                    "Chapter": {
                      "name": "Chapter 1",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n<blockquote style='position: relative; padding: 20px 20px;'><span style='position: absolute; top: 0; right: 5px; font-size: 80%; opacity: 0.4;'>CONFIDENTIAL</span>This is some highly confidential material which we want to remove when sharing with external parties.\n\nAnother *line*.\n\n# A title that should remain a title  \nYet another **line**.</blockquote>\n",
                      "number": [1],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis chapter will be removed if private is enabled\n",
                            "number": [1, 1],
                            "sub_items": [],
                            "path": "_chapter_1_sub.md",
                            "source_path": "_chapter_1_sub.md",
                            "parent_names": ["Chapter 1"]
                          }
                        }
                      ],
                      "path": "chapter_1.md",
                      "source_path": "chapter_1.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 2",
                      "content": "# Chapter 2\n\nThis chapter and it's subchapters will be removed if private is enabled\n",
                      "number": [2],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis will be removed if private is enabled because it's parent chapter is set to be removed.\n",
                            "number": [2, 1],
                            "sub_items": [],
                            "path": "chapter_2_sub.md",
                            "source_path": "chapter_2_sub.md",
                            "parent_names": ["Chapter 2"]
                          }
                        }
                      ],
                      "path": "_chapter_2.md",
                      "source_path": "_chapter_2.md",
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
    fn private_remove_chapters_run() {
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
                "mdbook_version": "0.4.32"
              },
              {
                "sections": [
                  {
                    "Chapter": {
                      "name": "Chapter 1",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n<!--private\nThis is some highly confidential material which we want to remove when sharing with external parties.\n\nAnother *line*.\n\n# A title that should remain a title  \nYet another **line**.\n-->\n",
                      "number": [1],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis chapter will be removed if private is enabled\n",
                            "number": [1, 1],
                            "sub_items": [],
                            "path": "_chapter_1_sub.md",
                            "source_path": "_chapter_1_sub.md",
                            "parent_names": ["Chapter 1"]
                          }
                        }
                      ],
                      "path": "chapter_1.md",
                      "source_path": "chapter_1.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 2",
                      "content": "# Chapter 2\n\nThis chapter and it's subchapters will be removed if private is enabled\n",
                      "number": [2],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis will be removed if private is enabled because it's parent chapter is set to be removed.\n",
                            "number": [2, 1],
                            "sub_items": [],
                            "path": "chapter_2_sub.md",
                            "source_path": "chapter_2_sub.md",
                            "parent_names": ["Chapter 2"]
                          }
                        }
                      ],
                      "path": "_chapter_2.md",
                      "source_path": "_chapter_2.md",
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
                "mdbook_version": "0.4.32"
              },
              {
                "sections": [
                  {
                    "Chapter": {
                      "name": "Chapter 1",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n",
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
    fn private_remove_chapters_section_numbers_run() {
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
                "mdbook_version": "0.4.32"
              },
              {
                "sections": [
                  { 
                    "Chapter": {
                      "name": "Intro",
                      "content": "# Intro\n\nIntroduction prefix chapter\n\n<!--private\nSecret stuff\n-->\n",
                      "number": null,
                      "sub_items": [],
                      "path": "intro.md",
                      "source_path": "intro.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 1",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n<!--private\nThis is some highly confidential material which we want to remove when sharing with external parties.\n\nAnother *line*.\n\n# A title that should remain a title  \nYet another **line**.\n-->\n",
                      "number": [1],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis chapter will be removed if private is enabled\n",
                            "number": [1, 1],
                            "sub_items": [],
                            "path": "_chapter_1_sub_1.md",
                            "source_path": "_chapter_1_sub.md",
                            "parent_names": ["Chapter 1"]
                          }
                        },
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "",
                            "number": [1, 2],
                            "sub_items": [],
                            "path": "chapter_1_sub_2.md",
                            "source_path": "chapter_1_sub_2.md",
                            "parent_names": ["Chapter 1"]
                          }
                        }
                      ],
                      "path": "chapter_1.md",
                      "source_path": "chapter_1.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 2",
                      "content": "# Chapter 2\n\nThis chapter and it's subchapters will be removed if private is enabled\n",
                      "number": [2],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "# Subchapter\n\nThis will be removed if private is enabled because it's parent chapter is set to be removed.\n",
                            "number": [2, 1],
                            "sub_items": [],
                            "path": "chapter_2_sub.md",
                            "source_path": "chapter_2_sub.md",
                            "parent_names": ["Chapter 2"]
                          }
                        }
                      ],
                      "path": "_chapter_2.md",
                      "source_path": "_chapter_2.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 3",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n\n",
                      "number": [3],
                      "sub_items": [],
                      "path": "chapter_3.md",
                      "source_path": "chapter_3.md",
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
                "mdbook_version": "0.4.32"
              },
              {
                "sections": [
                  {
                    "Chapter": {
                      "name": "Intro",
                      "content": "# Intro\n\nIntroduction prefix chapter\n\n",
                      "number": null,
                      "sub_items": [],
                      "path": "intro.md",
                      "source_path": "intro.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 1",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n",
                      "number": [1],
                      "sub_items": [
                        {
                          "Chapter": {
                            "name": "Sub chapter",
                            "content": "",
                            "number": [1, 1],
                            "sub_items": [],
                            "path": "chapter_1_sub_2.md",
                            "source_path": "chapter_1_sub_2.md",
                            "parent_names": ["Chapter 1"]
                          }
                        }
                      ],
                      "path": "chapter_1.md",
                      "source_path": "chapter_1.md",
                      "parent_names": []
                    }
                  },
                  {
                    "Chapter": {
                      "name": "Chapter 3",
                      "content": "# Chapter 1\n\nThis chapter will always be present\n\n\n",
                      "number": [2],
                      "sub_items": [],
                      "path": "chapter_3.md",
                      "source_path": "chapter_3.md",
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
