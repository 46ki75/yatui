# Rich Research Survey

## Evidence Header

```text
Research date: 2026-07-16
Project version: Rich 15.0.0
Project revision: 6ac483cbea39cab124dfd3483bba70ffafb71050 (tag v15.0.0)
Repository: https://github.com/Textualize/rich
Documentation version: Read the Docs stable; accessed 2026-07-16; source pinned to v15.0.0
Primary platform examined: Tagged source, tests, and documentation on Linux; no physical terminal reproduction
Report depth: Brief survey
```

**Project and version:** Rich is a typed Python library for styled terminal text and
structured presentation. PyPI and GitHub identify `15.0.0` as the current stable
release on the research date, released 2026-04-12. The tagged package requires
Python `>=3.9.0` and advertises Python 3.9 through 3.14; the release notes explicitly
drop Python 3.8. The stable documentation and tagged README still say 3.8, so the
release metadata is the versioned support authority ([PyPI metadata](https://pypi.org/project/rich/15.0.0/),
[release and commit](https://github.com/Textualize/rich/releases/tag/v15.0.0),
[tagged `pyproject.toml`](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/pyproject.toml)).

**Category and scope:** Rich is a **presentation/terminal formatting toolkit**,
not a full-screen application framework. `Console` owns formatting to a file or
terminal: ANSI color/style generation, terminal detection, dimensions, wrapping,
and renderable dispatch through the Console Protocol. `Live` can refresh arbitrary
renderables, redirect stdout/stderr, hide the cursor, and optionally enter the
alternate screen; `Console.input()` is line input, not a keyboard-event, focus, or
mouse system ([Console docs](https://rich.readthedocs.io/en/stable/console.html),
[Console source](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/rich/console.py),
[Live source](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/rich/live.py)).

**Distinctive strength:** Rich makes high-quality bounded output unusually cheap.
`Text` and markup provide composable styles; `cells.py` measures terminal cell
widths and splits grapheme-like spans, including recent multi-codepoint emoji
support. `Table` accepts renderables and resizes or wraps columns. `Live` provides
refresh and vertical-overflow policies, while `Progress` supports multiple tasks,
custom columns, file readers, and background refresh. These are strong building
blocks for CLI reports, logs, downloads, and streaming status, without requiring a
domain UI tree ([Unicode implementation](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/rich/cells.py),
[Live documentation](https://rich.readthedocs.io/en/stable/live.html),
[Progress documentation](https://rich.readthedocs.io/en/stable/progress.html)).

**Important limitation or tradeoff:** This is a limitation relative to ArborUI's
full-screen requirement, not a defect in Rich's intended scope. Rich does not
provide retained interaction identity, focus and event routing, mouse protocols,
serialized model updates, application scheduling, or a prepared-frame/uncertain-
write recovery contract. `Live(screen=True)` supplies an alternate-screen display,
but not those semantics. The workaround is to build an input/runtime/lifecycle
layer around Rich or adopt its sister project Textual; the cost is owning the
missing interaction, terminal recovery, and application-test infrastructure
separately ([tagged README](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/README.md)).

**Testing observation:** The pinned repository has focused pytest coverage for
`Console`, `Live`, `Progress`, `Table`, Unicode cells, and platform renderers.
`Console.capture()`, `StringIO` files, fixed dimensions, `record=True`, and text or
HTML/SVG exports support deterministic tests of the production render path. The
`Live` tests exercise captured ANSI output and disable auto-refresh for repeatable
cases. I found no complete application interaction harness or PTY recovery suite
in the tagged tests; input injection, focus, clocks, and physical-screen behavior
are therefore outside this testing boundary ([Console tests](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/tests/test_console.py),
[Live tests](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/tests/test_live.py),
[tagged test tree](https://github.com/Textualize/rich/tree/6ac483cbea39cab124dfd3483bba70ffafb71050/tests)).

**Relevance to ArborUI:** Adopt the lessons in renderable protocols, explicit cell
width and style handling, graceful non-terminal output, and capture-first visual
tests. Rich is useful as an adjacent presentation layer or comparison point, not
as evidence that a full-screen runtime is unnecessary. Do not infer from its
narrower scope that Rich is weak at rendering, that `screen=True` is a complete TUI
framework, or that logical Unicode tests prove physical terminal correctness.

**Best dated sources:** All sources were accessed 2026-07-16. The [PyPI release
page](https://pypi.org/project/rich/15.0.0/) and [GitHub release](https://github.com/Textualize/rich/releases/tag/v15.0.0)
are dated 2026-04-12; the [tagged package metadata](https://github.com/Textualize/rich/blob/6ac483cbea39cab124dfd3483bba70ffafb71050/pyproject.toml),
[tagged implementation and tests](https://github.com/Textualize/rich/tree/6ac483cbea39cab124dfd3483bba70ffafb71050),
and [stable documentation](https://rich.readthedocs.io/en/stable/introduction.html) provide the
versioned and current-scope evidence.
