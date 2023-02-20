# mdbook-private

*A preprocessor for [mdbook](https://github.com/rust-lang-nursery/mdBook) that allows for private sections to be hidden or kept.*

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


