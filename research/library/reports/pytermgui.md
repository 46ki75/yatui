# PyTermGUI Research Survey

## Evidence Header

```text
Research date: 2026-07-16
Project version: PyTermGUI 7.7.4
Project revision: 8697607f6280071895e8b3329545039399a61c39
Repository: https://github.com/bczsalba/pytermgui
Documentation version: MkDocs source at v7.7.4; live docs checked 2026-07-16
Primary platform examined: Tagged source, tests, and docs on Linux; no physical terminal reproduction
Report depth: Brief survey
```

The version-specific [PyPI metadata](https://pypi.org/pypi/pytermgui/7.7.4/json) and
[GitHub release](https://github.com/bczsalba/pytermgui/releases/tag/v7.7.4) identify
7.7.4 as the latest stable release on the research date, released 2025-03-31. The
tag resolves to the revision above. The tagged README says that Shade40 is now the
primary development focus, so this is a production/stable release with a stated
shifted maintenance direction.

**Project and version:** PyTermGUI is a typed, MIT-licensed Python package. Its
[package metadata](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pyproject.toml#L5-L45)
requires Python `>=3.8`, depends on `wcwidth` and `typing_extensions`, and offers
optional YAML styling. PyPI classifiers list Python 3.8-3.11 and macOS/POSIX Linux;
the pinned CI also tests Ubuntu, Windows, and macOS on Python 3.8-3.10.

**Category and scope:** This is a **Python TUI toolkit/framework** at its actual
scope: a retained widget, window, and layout system with a synchronous
[`WindowManager`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/window_manager/manager.py#L45-L59)
input loop and a separate compositor thread. `Widget` and `Container` objects
return line-oriented output; `Layout` supplies static, relative, and auto-sized
slots. Focus is window-level plus container selection; keyboard and mouse input
cascade through widgets. Modal `Window` objects, alerts, and toasts provide
overlays. `WindowManager` owns the full-screen alternate-buffer mode, while
[`inline()`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/widgets/inline.py#L26-L35)
runs the same widgets as prompts. It has no mandatory async runtime or serialized
effect queue. Display width uses [`wcwidth`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/regex.py#L60-L75),
but [`InputField`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/widgets/input_field.py#L85-L101)
rejects wide Unicode characters, limiting Unicode-heavy text input.

**Distinctive strength:** PyTermGUI makes desktop-like windows, focus traversal,
mouse dragging, modal overlays, animation, and forms compact to express. `auto`
converts strings, tuples, mappings, and lists into widgets; TIM markup plus YAML or
Python styles provide a concise presentation layer. The same widget API supports
full-screen and inline applications, with color degradation and `NO_COLOR` support
as practical terminal fallbacks.

**Important limitation or tradeoff:** **Classification: limitation relative to
ArborUI's recoverable runtime.** The pinned
[compositor](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/window_manager/compositor.py#L233-L281)
redraws full positioned lines rather than exposing a prepared-frame transaction,
and [`Terminal.write`/`flush`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/term.py#L499-L601)
return no applied, deferred, or unknown outcome. External updates are documented
as application-created threads that mutate widget state; there is no runtime-level
ordering, cancellation, or backpressure contract. `WindowManager` uses an
alternate-buffer context for ordinary cleanup, but
[`inline()`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/widgets/inline.py#L49-L116)
restores echo and cursor state only after normal loop exit. The workaround is an
application-owned queue/synchronization policy, explicit redraw/restart handling,
and PTY lifecycle tests. The cost is that uncertain output, concurrent updates,
and recovery after a failure remain outside the framework contract.

**Testing observation:** The pinned
[suite](https://github.com/bczsalba/pytermgui/tree/8697607f6280071895e8b3329545039399a61c39/tests)
tests layout, markup/parser round trips, styles, animations, and HTML/SVG export
using fixed-size or recording terminals. `feed()` can emulate input, but no
complete `WindowManager` application harness, virtual terminal, PTY lifecycle
suite, partial-write fault injection, or controlled clock was found in the tagged
[tests and workflow](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/.github/workflows/pytest.yml#L10-L40).
Tests therefore cover useful components and captured logical output, not the
complete production interaction and recovery path.

**Relevance to ArborUI:** PyTermGUI is a useful precedent for compact retained
widgets, explicit full-screen versus inline modes, cell-width-aware rendering, and
context-managed terminal cleanup. ArborUI should not treat its synchronous thread
pattern, wide-character input restriction, full redraw path, or captured exports as
evidence for serialized effects, physical-screen recovery, or deterministic
complete-application testing.

**Best sources:** All sources were accessed on 2026-07-16.

| Claim | Primary immutable source | Source date |
| --- | --- | --- |
| Release, stable version, and exact revision | [PyPI 7.7.4 metadata](https://pypi.org/pypi/pytermgui/7.7.4/json); [v7.7.4 release](https://github.com/bczsalba/pytermgui/releases/tag/v7.7.4); [pinned commit](https://github.com/bczsalba/pytermgui/commit/8697607f6280071895e8b3329545039399a61c39) | 2025-03-31 |
| Scope, Python/platform metadata, and maintenance direction | [tagged `pyproject.toml`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pyproject.toml); [tagged README](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/README.md#L31-L65); [CI workflow](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/.github/workflows/pytest.yml#L10-L40) | 2025-03-31 |
| Widgets, input, layout, overlays, lifecycle, and rendering | [`WindowManager`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/window_manager/manager.py#L130-L205); [`Compositor`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/window_manager/compositor.py#L233-L281); [`alt_buffer`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/context_managers.py#L72-L105); [custom-widget docs](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/docs/widgets/custom.md#L7-L19) | 2025-03-31 |
| Unicode and testing boundary | [`wcwidth` measurement](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/regex.py#L60-L75); [`InputField`](https://github.com/bczsalba/pytermgui/blob/8697607f6280071895e8b3329545039399a61c39/pytermgui/widgets/input_field.py#L85-L101); [tagged tests](https://github.com/bczsalba/pytermgui/tree/8697607f6280071895e8b3329545039399a61c39/tests) | 2025-03-31 |
