use yatui_core::{CursorState, Style};
use yatui_layout::LayoutStyle;

use crate::{EventContext, EventPhase, Key, UiEvent};

/// Stable category used to determine whether retained state is compatible.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WidgetKind {
    /// A flex container with no intrinsic visual content.
    Container,
    /// Borrowed text content.
    Text,
    /// A third-party widget category.
    Custom(&'static str),
}

#[derive(Clone, Copy, Debug)]
enum Content<'a> {
    Empty,
    Text(&'a str),
}

/// Frame-local declarative UI node.
///
/// Borrowed content is used only during synchronous reconciliation, layout,
/// and painting. It is never copied into the retained tree.
pub struct Element<'a, Message> {
    key: Option<Key>,
    kind: WidgetKind,
    layout: LayoutStyle,
    style: Style,
    content: Content<'a>,
    children: Vec<Self>,
    handlers: Vec<EventHandler<'a, Message>>,
    interactive: bool,
    focusable: bool,
    focus_scope: bool,
    focus_order: Option<i32>,
    cursor: Option<CursorState>,
}

struct EventHandler<'a, Message> {
    phase: EventPhase,
    callback: Box<HandlerCallback<'a, Message>>,
}

type HandlerCallback<'a, Message> = dyn Fn(&UiEvent, &mut EventContext<'_, Message>) + 'a;

impl<'a, Message> Element<'a, Message> {
    /// Creates an empty container from ordered children.
    #[must_use]
    pub fn container(children: impl IntoIterator<Item = Self>) -> Self {
        Self {
            key: None,
            kind: WidgetKind::Container,
            layout: LayoutStyle::default(),
            style: Style::default(),
            content: Content::Empty,
            children: children.into_iter().collect(),
            handlers: Vec::new(),
            interactive: false,
            focusable: false,
            focus_scope: false,
            focus_order: None,
            cursor: None,
        }
    }

    /// Creates a borrowed text leaf.
    #[must_use]
    pub fn text(text: &'a str) -> Self {
        Self {
            key: None,
            kind: WidgetKind::Text,
            layout: LayoutStyle::default(),
            style: Style::default(),
            content: Content::Text(text),
            children: Vec::new(),
            handlers: Vec::new(),
            interactive: false,
            focusable: false,
            focus_scope: false,
            focus_order: None,
            cursor: None,
        }
    }

    /// Creates a custom node without intrinsic content.
    #[must_use]
    pub fn custom(kind: &'static str, children: impl IntoIterator<Item = Self>) -> Self {
        let mut element = Self::container(children);
        element.kind = WidgetKind::Custom(kind);
        element
    }

    /// Assigns explicit stable identity.
    #[must_use]
    pub fn key(mut self, key: impl Into<Key>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Assigns layout behavior.
    #[must_use]
    pub const fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    /// Assigns visual cell styling.
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Registers an ephemeral handler for one dispatch phase.
    #[must_use]
    pub fn on_event(
        mut self,
        phase: EventPhase,
        handler: impl Fn(&UiEvent, &mut EventContext<'_, Message>) + 'a,
    ) -> Self {
        self.handlers.push(EventHandler {
            phase,
            callback: Box::new(handler),
        });
        self.interactive = true;
        self
    }

    /// Enables or disables spatial hit testing for this element.
    #[must_use]
    pub const fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Enables or disables keyboard focus for this element.
    #[must_use]
    pub const fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        if focusable {
            self.interactive = true;
        }
        self
    }

    /// Marks this element as a focus scope, such as an overlay or dialog.
    #[must_use]
    pub const fn focus_scope(mut self, focus_scope: bool) -> Self {
        self.focus_scope = focus_scope;
        self
    }

    /// Sets explicit traversal order within a focus scope.
    #[must_use]
    pub const fn focus_order(mut self, order: i32) -> Self {
        self.focus_order = Some(order);
        self
    }

    /// Sets a terminal cursor intent local to this element's border box.
    #[must_use]
    pub const fn cursor(mut self, cursor: CursorState) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Returns explicit identity, if present.
    #[must_use]
    pub const fn explicit_key(&self) -> Option<&Key> {
        self.key.as_ref()
    }

    /// Returns the widget category.
    #[must_use]
    pub const fn kind(&self) -> WidgetKind {
        self.kind
    }

    /// Returns the layout style.
    #[must_use]
    pub const fn layout_style(&self) -> LayoutStyle {
        self.layout
    }

    /// Returns the visual style.
    #[must_use]
    pub const fn visual_style(&self) -> Style {
        self.style
    }

    /// Returns ordered child declarations.
    #[must_use]
    pub fn children(&self) -> &[Self] {
        &self.children
    }

    pub(crate) fn handlers(
        &self,
        phase: EventPhase,
    ) -> impl Iterator<Item = &HandlerCallback<'a, Message>> {
        self.handlers
            .iter()
            .filter(move |handler| handler.phase == phase)
            .map(|handler| handler.callback.as_ref())
    }

    pub(crate) const fn is_interactive(&self) -> bool {
        self.interactive
    }

    pub(crate) const fn is_focusable(&self) -> bool {
        self.focusable
    }

    pub(crate) const fn is_focus_scope(&self) -> bool {
        self.focus_scope
    }

    pub(crate) const fn explicit_focus_order(&self) -> Option<i32> {
        self.focus_order
    }

    pub(crate) const fn cursor_intent(&self) -> Option<CursorState> {
        self.cursor
    }

    pub(crate) const fn text_content(&self) -> Option<&'a str> {
        match self.content {
            Content::Empty => None,
            Content::Text(text) => Some(text),
        }
    }
}
