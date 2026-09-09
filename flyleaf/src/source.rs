//! The document as text, read-only: what a save would write, with the row
//! the tree is working in highlighted.
//!
//! Half the value of a structure-aware editor is seeing what its decisions
//! come to, and the tree hides what a person who hand-edits TOML cares
//! about: an inline table against a `[header]`, a quoting style, where a
//! comment went. This is where those are visible. Read-only on purpose;
//! `PLAN.md` records why an editable pane is a different product.
//!
//! Drawn as one label per line inside a `ScrollArea` that lays out only the
//! visible rows, which every line being the same monospace height allows.
//! A `TextEdit` would give selection and copy, and lays out the whole text
//! every frame; for a `Cargo.lock` of five thousand lines the rows win.

use std::ops::Range;

use egui::{self, Ui};

fn scrolled_id() -> egui::Id {
    egui::Id::new("flyleaf::source::scrolled")
}

/// Draw the text with a range of lines highlighted.
///
/// The highlight is scrolled into view the frame it changes and left alone
/// after, so that somebody reading elsewhere in the text is not pulled back
/// every frame.
pub fn source(ui: &mut Ui, text: &str, highlight: Option<&Range<usize>>) {
    let lines: Vec<&str> = text.lines().collect();
    let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
    let digits = lines.len().max(1).to_string().len();

    let scrolled: Option<Option<Range<usize>>> = ui.data(|d| d.get_temp(scrolled_id()));
    let mut area = egui::ScrollArea::both().auto_shrink([false, false]);
    if scrolled.as_ref().map(Option::as_ref) != Some(highlight) {
        if let Some(range) = highlight {
            // The highlighted lines a third of the way down, which is where
            // an eye looks for what it was just shown. A line number is
            // exact in an f32 up to sixteen million lines.
            #[allow(clippy::cast_precision_loss)]
            let y = row_height * range.start as f32 - ui.available_height() / 3.0;
            area = area.vertical_scroll_offset(y.max(0.0));
        }
        ui.data_mut(|d| d.insert_temp(scrolled_id(), highlight.cloned()));
    }

    area.show_rows(ui, row_height, lines.len(), |ui, rows| {
        for i in rows {
            let lit = highlight.is_some_and(|r| r.contains(&i));
            let background = ui.painter().add(egui::Shape::Noop);
            let drawn = ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                ui.label(
                    egui::RichText::new(format!("{:>digits$}", i + 1))
                        .monospace()
                        .weak(),
                );
                ui.add(
                    egui::Label::new(egui::RichText::new(lines[i]).monospace())
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
            });
            if lit {
                ui.painter().set(
                    background,
                    egui::Shape::rect_filled(
                        drawn.response.rect.expand2(egui::vec2(4.0, 0.0)),
                        2.0,
                        ui.visuals().selection.bg_fill.gamma_multiply(0.25),
                    ),
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::source;

    /// One frame at a fixed size: every text drawn, in order, and how many
    /// filled rectangles were painted.
    fn drawn(text: &str, highlight: Option<&std::ops::Range<usize>>) -> (Vec<String>, usize) {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600.0, 2000.0),
            )),
            ..Default::default()
        };
        let mut output = ctx.run_ui(input, |ui| source(ui, text, highlight));
        let shapes = std::mem::take(&mut output.shapes);
        output.drop_without_applying_deltas();
        let texts = shapes
            .iter()
            .filter_map(|c| match &c.shape {
                egui::Shape::Text(t) => Some(t.galley.text().to_owned()),
                _ => None,
            })
            .collect();
        let rects = shapes
            .iter()
            .filter(|c| matches!(c.shape, egui::Shape::Rect(_)))
            .count();
        (texts, rects)
    }

    /// Every line is drawn with its number, in order, and a blank line is a
    /// line: a pane that skipped one would misnumber everything after it.
    #[test]
    fn every_line_is_drawn_and_numbered() {
        let (texts, _) = drawn("a = 1\n\n[t]\nb = 2\n", None);
        assert_eq!(texts, ["1", "a = 1", "2", "", "3", "[t]", "4", "b = 2"]);
    }

    /// A highlighted range paints one rectangle per line in it and none
    /// otherwise, which is the whole of the sync as the eye sees it.
    #[test]
    fn the_highlight_is_painted_under_its_lines() {
        let text = "a = 1\nb = 2\nc = 3\n";
        let (_, plain) = drawn(text, None);
        let (_, lit) = drawn(text, Some(&(1..3)));
        assert_eq!(lit, plain + 2, "two lines lit");
    }

    /// Only the visible rows are laid out: a document longer than the
    /// screen draws a fraction of its lines, which is what keeps a lockfile
    /// cheap to show.
    #[test]
    fn only_the_visible_rows_are_laid_out() {
        use std::fmt::Write as _;
        let mut text = String::new();
        for i in 0..5000 {
            writeln!(text, "k{i} = {i}").expect("writing to a String");
        }
        let (texts, _) = drawn(&text, None);
        let lines = texts.iter().filter(|t| t.starts_with('k')).count();
        assert!(lines > 10 && lines < 500, "{lines} of 5000 lines drawn");
    }
}
