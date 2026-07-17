use arborui_core::{Modifier, Style};
use arborui_layout::LayoutStyle;
use arborui_ui::{Element, EventPhase};

use crate::activation::handle_activation;

/// Creates a focusable button builder with a borrowed text label.
#[must_use]
pub fn button<'a, Message>(
    label: &'a str,
    on_press: impl Fn() -> Message + 'a,
) -> Button<'a, Message>
where
    Message: 'a,
{
    Button::new(label, on_press)
}

/// Builder for a focusable controlled button.
///
/// Activation messages come from a repeatable factory, so `Message` does not
/// need to implement [`Clone`].
pub struct Button<'a, Message> {
    label: &'a str,
    on_press: Box<dyn Fn() -> Message + 'a>,
    style: Style,
    label_style: Style,
    focus_style: Style,
    layout: LayoutStyle,
    focus_order: Option<i32>,
}

impl<'a, Message: 'a> Button<'a, Message> {
    /// Creates a button with `label` and an activation message factory.
    #[must_use]
    pub fn new(label: &'a str, on_press: impl Fn() -> Message + 'a) -> Self {
        Self {
            label,
            on_press: Box::new(on_press),
            style: Style::default(),
            label_style: Style::default(),
            focus_style: Style::new().add_modifiers(Modifier::REVERSED),
            layout: LayoutStyle::default(),
            focus_order: None,
        }
    }

    /// Sets the button container style.
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the label style.
    #[must_use]
    pub const fn label_style(mut self, style: Style) -> Self {
        self.label_style = style;
        self
    }

    /// Sets the style applied while the button owns keyboard focus.
    #[must_use]
    pub const fn focus_style(mut self, style: Style) -> Self {
        self.focus_style = style;
        self
    }

    /// Sets the button layout properties.
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

    /// Builds the declarative button element.
    #[must_use]
    pub fn build(self) -> Element<'a, Message> {
        let on_press = self.on_press;
        let mut element = Element::custom(
            "button",
            [Element::text(self.label).style(self.label_style)],
        )
        .layout(self.layout)
        .style(self.style)
        .focus_style(self.focus_style)
        .focusable(true)
        .on_event(EventPhase::Target, move |event, context| {
            handle_activation(event, context, &on_press);
        });
        if let Some(order) = self.focus_order {
            element = element.focus_order(order);
        }
        element
    }
}
