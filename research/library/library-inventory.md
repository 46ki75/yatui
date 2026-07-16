# TUI Library Research Inventory

Research baseline: 2026-07-16

This inventory classifies the projects in `libraries-raw.md` before comparing
them with ArborUI. The category describes the primary responsibility examined,
not a quality ranking. Some projects have multiple roles; the scope note names
the role most relevant to the report. Report depth follows
[`library-research-strategy.md`](library-research-strategy.md).

| Project | Language | Category | Scope examined | Report |
| --- | --- | --- | --- | --- |
| [notcurses](https://github.com/dankamongmen/notcurses) | C, Python bindings | Terminal substrate; rendering | Native terminal capabilities, planes, compositing, media, and bindings | [Standard](reports/notcurses.md) |
| [pytermgui](https://github.com/bczsalba/pytermgui) | Python | Application framework; widgets | Python widgets, layout, event loop, and terminal ownership | [Brief](reports/pytermgui.md) |
| [Python Prompt Toolkit](https://github.com/prompt-toolkit/python-prompt-toolkit) | Python | Presentation and input toolkit | Interactive line editing, prompts, completion, and full-screen primitives | [Standard](reports/python-prompt-toolkit.md) |
| [Rich](https://github.com/Textualize/rich) | Python | Presentation toolkit | Rich text, tables, progress, live displays, and terminal rendering | [Brief](reports/rich.md) |
| [Textual](https://github.com/Textualize/textual) | Python | Application framework | Retained widgets, CSS layout, messages, workers, and testing | [Deep](reports/textual.md) |
| [Bubble Tea](https://github.com/charmbracelet/bubbletea) | Go | Application framework | Elm-style model, messages, commands, rendering, and lifecycle | [Deep](reports/bubble-tea.md) |
| [gocui](https://github.com/jroimartin/gocui) | Go | Rendering and widget library | Retained views, layout managers, keybindings, and main loop | [Brief](reports/gocui.md) |
| [PTerm](https://github.com/pterm/pterm) | Go | Presentation toolkit | Cross-platform formatted output, progress, tables, and prompts | [Brief](reports/pterm.md) |
| [tview](https://github.com/rivo/tview) | Go | Rendering and widget library | Retained primitives, layouts, focus, pages, and tcell integration | [Standard](reports/tview.md) |
| [tcell](https://github.com/gdamore/tcell) | Go | Terminal substrate | Terminal capabilities, input decoding, cells, screens, and simulation | [Standard](reports/tcell.md) |
| [FTXUI](https://github.com/ArthurSonzogni/FTXUI) | C++ | Rendering and widget library; application framework | Functional components, layout, event handling, and screen renderers | [Standard](reports/ftxui.md) |
| [Spectre.Console](https://github.com/spectreconsole/spectre.console) | .NET | Presentation toolkit | Renderables, live displays, progress, tables, and prompts | [Brief](reports/spectre-console.md) |
| [Terminal.Gui](https://github.com/gui-cs/Terminal.Gui) | .NET | Application framework | Retained views, focus, navigation, menus, dialogs, and drivers | [Standard](reports/terminal-gui.md) |
| [iocraft](https://github.com/ccbrown/iocraft) | Rust | Application framework | Declarative Rust components, hooks, layout, events, and terminal runtime | [Deep](reports/iocraft.md) |
| [Ratatui](https://github.com/ratatui/ratatui) | Rust | Rendering and widget library | Immediate-mode buffers, layout, widgets, backends, and logical-screen tests | [Deep](reports/ratatui.md) |
| [blessed](https://github.com/chjj/blessed) | Node.js | Application framework; rendering and widgets | Retained screen tree, widgets, event handling, and ANSI output | [Standard](reports/blessed.md) |
| [gum](https://github.com/charmbracelet/gum) | Go | Presentation and shell toolkit | Composable shell commands for prompts, selectors, spinners, and formatting | [Brief](reports/gum.md) |
| [Ink](https://github.com/vadimdemedes/ink) | Node.js | Application framework | React reconciliation, Yoga layout, ANSI rendering, and Node streams | [Deep](reports/ink.md) |
| [OpenTUI](https://github.com/anomalyco/opentui) | TypeScript, Zig | Application framework; terminal substrate | Retained native renderer, cell compositor, input parser, and React/Solid bindings | [Deep](reports/opentui.md) |

## Reading Notes

- Deep reports are the main evidence for direct architectural alternatives to
  ArborUI: Textual, Bubble Tea, iocraft, Ink, OpenTUI, and Ratatui.
- Standard reports cover important subsystems or mature alternatives without
  the same source depth as the deep dives.
- Brief surveys cover adjacent presentation tools and intentionally narrow
  libraries. Their lack of full-application machinery is not treated as a defect
  unless the project claims that scope.
- Version, revision, maintenance, and evidence status belong to the individual
  report. This table is an index, not a replacement for those baselines.
