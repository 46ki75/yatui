use yatui_core::{Point, Rect};

use crate::{Buffer, BufferError, CellContent};

/// How empty cells on a surface affect lower surfaces.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Opacity {
    /// Empty cells leave lower surfaces unchanged.
    #[default]
    Transparent,
    /// Empty cells overwrite lower surfaces.
    Opaque,
}

/// An independently positioned and clipped cell buffer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Surface {
    /// Surface cell contents.
    pub buffer: Buffer,
    /// Surface origin in target coordinates.
    pub position: Point,
    /// Visible region in local surface coordinates.
    pub clip: Rect,
    /// Paint ordering; larger values appear above smaller values.
    pub z_index: i32,
    /// Empty-cell composition behavior.
    pub opacity: Opacity,
}

impl Surface {
    /// Creates a transparent surface at the origin.
    #[must_use]
    pub fn new(buffer: Buffer) -> Self {
        let clip = buffer.bounds();
        Self {
            buffer,
            position: Point::ORIGIN,
            clip,
            z_index: 0,
            opacity: Opacity::Transparent,
        }
    }
}

/// Composes ordered surfaces into a target buffer.
pub struct Compositor;

impl Compositor {
    /// Composes surfaces in ascending z-index order.
    pub fn compose(target: &mut Buffer, surfaces: &[Surface]) -> Result<(), BufferError> {
        let mut ordered: Vec<_> = surfaces.iter().enumerate().collect();
        ordered.sort_by_key(|(index, surface)| (surface.z_index, *index));

        for (_, surface) in ordered {
            compose_surface(target, surface)?;
        }
        Ok(())
    }
}

fn compose_surface(target: &mut Buffer, surface: &Surface) -> Result<(), BufferError> {
    let Some(clip) = surface.clip.intersection(surface.buffer.bounds()) else {
        return Ok(());
    };

    for y in clip.y..clip.bottom() {
        for x in clip.x..clip.right() {
            let source_point = Point::new(x, y);
            let Some(cell) = surface.buffer.get(source_point) else {
                continue;
            };
            let target_point = surface.position.translated(x, y);
            match cell.content {
                CellContent::Empty if surface.opacity == Opacity::Opaque => {
                    if target.bounds().contains(target_point) {
                        target.set_empty(target_point, cell.style)?;
                    }
                }
                CellContent::Empty | CellContent::Continuation { .. } => {}
                CellContent::Grapheme { id, width } => {
                    let source_end = source_point.x.saturating_add(i32::from(width) - 1);
                    let target_end = target_point.x.saturating_add(i32::from(width) - 1);
                    if clip.contains(Point::new(source_end, source_point.y))
                        && target.bounds().contains(target_point)
                        && target
                            .bounds()
                            .contains(Point::new(target_end, target_point.y))
                    {
                        target.set_grapheme(target_point, id, width, cell.style, cell.hyperlink)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use yatui_core::{Color, Size, Style};

    use super::*;
    use crate::GraphemeId;

    #[test]
    fn higher_surfaces_overwrite_lower_surfaces() -> Result<(), BufferError> {
        let mut lower = Buffer::new(Size::new(1, 1));
        lower.set_empty(Point::ORIGIN, Style::new().background(Color::Blue))?;
        let mut upper = Buffer::new(Size::new(1, 1));
        upper.set_grapheme(
            Point::ORIGIN,
            GraphemeId::from_test_value(1),
            1,
            Style::new().foreground(Color::White),
            None,
        )?;
        let surfaces = [Surface::new(lower), Surface::new(upper)];
        let mut target = Buffer::new(Size::new(1, 1));

        Compositor::compose(&mut target, &surfaces)?;

        assert!(matches!(
            target.get(Point::ORIGIN).map(|cell| cell.content),
            Some(CellContent::Grapheme { .. })
        ));
        Ok(())
    }
}
