# mdbook-private

[![build](https://github.com/RealAtix/mdbook-private/actions/workflows/build.yml/badge.svg)](https://github.com/RealAtix/mdbook-private/actions/workflows/build.yml)
[![crate.io](https://img.shields.io/crates/v/mdbook-private)](https://crates.io/crates/mdbook-private)
[![downloads](https://img.shields.io/crates/d/mdbook-private)](https://crates.io/crates/mdbook-private)
[![license](https://img.shields.io/github/license/RealAtix/mdbook-private)](LICENSE)

> A preprocessor for [mdbook](https://github.com/rust-lang-nursery/mdBook) that allows for private sections to be defined and hidden or kept.

## Usage

**Installation**
```sh
cargo install mdbook-private
```

**Then add it to your book.toml**
```toml
# Default options
[preprocessor.private]
remove = false
style = true
notice = "CONFIDENTIAL"
```

**Options**
- remove (boolean): remove or keep the sections defined as private
- style (boolean): style the sections (if kept), currently using the blockquote css
- notice (string): add a notice to the styled sections in the top right corner

**Usage in Markdown**
```markdown
<!--private
This is some highly confidential material which we want to hide when sharing with external parties.

Another *line*.

# A title that should remain a title  
Yet another **line**.
-->
```

<img width="771" alt="Example output" src="https://user-images.githubusercontent.com/4161235/220068655-96b89372-784e-4a12-8ef0-8f15b7d0c557.png">

