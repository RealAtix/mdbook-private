# mdbook-private

[![build](https://github.com/RealAtix/mdbook-private/actions/workflows/build.yml/badge.svg)](https://github.com/RealAtix/mdbook-private/actions/workflows/build.yml)
[![crate.io](https://img.shields.io/crates/v/mdbook-private)](https://crates.io/crates/mdbook-private)
[![downloads](https://img.shields.io/crates/d/mdbook-private)](https://crates.io/crates/mdbook-private)
[![license](https://img.shields.io/github/license/RealAtix/mdbook-private)](LICENSE)

> An [mdbook](https://github.com/rust-lang-nursery/mdBook) preprocessor for defining and optionally hiding private sections and chapters in your book.

## Usage

**Installation**
```sh
cargo install mdbook-private
```

**Configuration in book.toml**
```toml
# Default options
[preprocessor.private]
remove = false
style = true
notice = "CONFIDENTIAL"
chapter-prefix = "_"
```

**Options Explained**
- `remove` (boolean): Determines whether to remove or retain sections marked as private.
- `style` (boolean): Styles the private sections (when retained) using blockquote CSS.
- `notice` (string): Adds a notice to styled sections at the top right corner.
- `chapter-prefix` (string): If the `remove` option is active, chapters with filenames prefixed with this value will be excluded.

**Markdown Usage**

For a hands-on example, explore the `example-book`.

```
# Summary

- [Chapter 1](./chapter_1.md)
  - [Sub chapter](./_chapter_1_sub.md)
- [Chapter 2](./_chapter_2.md)
  - [Sub chapter](./chapter_2_sub.md)
```
Note: With the `remove` option enabled, only "Chapter 1" will be retained.

---

```markdown
<!--private
This is some highly confidential material which we want to hide when sharing with external parties.

Another *line*.

# A title that should remain a title  
Yet another **line**.
-->
```

![Example output](https://user-images.githubusercontent.com/4161235/220068655-96b89372-784e-4a12-8ef0-8f15b7d0c557.png)
