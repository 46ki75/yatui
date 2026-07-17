use arborui_core::{Modifier, Style};
use arborui_layout::LayoutStyle;
use arborui_ui::{Element, EventPhase};

use crate::activation::handle_activation;

/// Creates a focusable controlled checkbox builder.
#[must_use]
pub fn checkbox<'a, Message>(
    label: &'a str,
    checked: bool,
    on_change: impl Fn(bool) -> Message + 'a,
) -> Checkbox<'a, Message>
where
    Message: 'a,
{
    Checkbox::new(label, checked, on_change)
}

/// Builder for a focusable controlled checkbox.
pub struct Checkbox<'a, Message> {
    label: &'a str,
    checked: bool,
    on_change: Box<dyn Fn(bool) -> Message + 'a>,
    style: Style,
    label_style: Style,
    focus_style: Style,
    layout: LayoutStyle,
    focus_order: Option<i32>,
}

impl<'a, Message: 'a> Checkbox<'a, Message> {
    /// Creates a checkbox borrowing its label and emitting the next checked state.
    #[must_use]
    pub fn new(label: &'a str, checked: bool, on_change: impl Fn(bool) -> Message + 'a) -> Self {
        Self {
            label,
            checked,
            on_change: Box::new(on_change),
            style: Style::default(),
            label_style: Style::default(),
            focus_style: Style::new().add_modifiers(Modifier::REVERSED),
            layout: LayoutStyle::default(),
            focus_order: None,
        }
    }

    /// Sets the checkbox container style.
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the marker and label style.
    #[must_use]
    pub const fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Sets the style applied while the checkbox owns keyboard focus.
    #[must_use]
    pub const fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = style;
        self
    }

    /// Sets the checkbox layout properties.
    #[must_use]
    pub const fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    /// Sets explicit focus traversal order.
    #[must_use]
    pub const fn focus_order(mut self, order: i32) -> Self {
        self.focus_order = Some(order);
        self
    }

    /// Builds the declarative checkbox element.
    #[must_use]
    pub fn build(self) -> Element<'a, Message> {
        let marker = if self.checked { "[x]" } else { "[ ]" };
        let next = !self.checked;
        let on_change = self.on_change;
        let mut element = Element::custom(
            "checkbox",
            [
                Element::text(marker).style(self.label_style),
                Element::text(" ").style(self.label_style),
                Element::text(self.label).style(self.label_style),
            ],
        )
        .layout(self.layout)
        .style(self.style)
        .focus_style(self.focus_style)
        .focusable(true)
        .on_event(EventPhase::Target, move |event, context| {
            handle_activation(event, context, || on_change(next));
        });
        if let Some(order) = self.focus_order {
            element = element.focus_order(order);
        }
        element
    }
}
