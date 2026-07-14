# Fuzz Corpus

The fuzz targets use bounded bytecode so corpus entries remain meaningful when
internal Rust types change.

## Targets

- `text_edit_sequences` interprets a length-prefixed initial string followed by
  insert, delete, movement, and selection operations. Its seeds cover ASCII,
  combining sequences, CJK text, ZWJ emoji, and regional indicators.
- `render_transactions` interprets five-byte frame operations containing size,
  signed paint coordinates, text selection, commit or discard, invalidation,
  and width-policy changes. Its seeds cover zero-area frames, resize, clipping,
  and Unicode-wide content.

When a failure is fixed, minimize it with `cargo fuzz tmin`, retain the input in
the matching corpus directory, and add a descriptive regression test to the
owning crate.
