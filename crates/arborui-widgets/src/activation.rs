use arborui_ui::{EventContext, KeyAction, PointerButton, PointerEventKind, UiEvent, UiKey};

pub(crate) fn handle_activation<Message>(
    event: &UiEvent,
    context: &mut EventContext<'_, Message>,
    message: impl FnOnce() -> Message,
) {
    match event {
        UiEvent::Pointer(pointer)
            if pointer.kind == PointerEventKind::Down(PointerButton::Primary) =>
        {
            context.capture_pointer();
            context.emit(message());
            context.mark_handled();
        }
        UiEvent::Pointer(pointer)
            if pointer.kind == PointerEventKind::Up(PointerButton::Primary) =>
        {
            context.release_pointer();
            context.mark_handled();
        }
        UiEvent::Key(key)
            if key.action == KeyAction::Press
                && matches!(key.key, UiKey::Enter | UiKey::Character(' ')) =>
        {
            context.emit(message());
            context.mark_handled();
        }
        _ => {}
    }
}
