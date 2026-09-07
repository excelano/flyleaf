//! Tommy Flyleaf, the application: a window around the widget.
//!
//! `flyleaf path/to/file.toml` opens that file and shows it as a tree. Edits
//! stay in memory and the window says so: saving arrives with open and
//! save-as, which is Phase 2 item 6 of `PROMPT.md`, and this shell exists so
//! that the widget can be run on its own before then. One optional argument,
//! no flags and no subcommands; on Windows this is a GUI-subsystem executable
//! and prints nothing, which is why an error is shown in the window rather
//! than written anywhere.
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use flyleaf_core::toml_edit::DocumentMut;

/// What the window shows.
enum Shown {
    /// No argument was given.
    Nothing,
    /// The file, parsed.
    Document { path: PathBuf, doc: DocumentMut },
    /// The file could not be read or was not TOML, and this is what was said.
    Failed { path: PathBuf, why: String },
}

/// Read and parse a file, with the reason where either fails.
///
/// `toml_edit`'s error carries the line and a caret under the column, which is
/// what somebody wants to see beside a file that would not open.
fn open(path: &Path) -> Result<DocumentMut, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    text.parse::<DocumentMut>().map_err(|e| e.to_string())
}

/// What the first argument opens, or what the window shows without one.
fn shown(arg: Option<PathBuf>) -> Shown {
    match arg {
        None => Shown::Nothing,
        Some(path) => match open(&path) {
            Ok(doc) => Shown::Document { path, doc },
            Err(why) => Shown::Failed { path, why },
        },
    }
}

/// The window's title: the file's name first, so that a task bar full of
/// windows reads as files rather than as a row of the same application.
fn title(shown: &Shown) -> String {
    match shown {
        Shown::Nothing => "Tommy Flyleaf".to_owned(),
        Shown::Document { path, .. } | Shown::Failed { path, .. } => {
            let name = path.file_name().map_or_else(
                || path.display().to_string(),
                |n| n.to_string_lossy().into_owned(),
            );
            format!("{name} \u{2014} Tommy Flyleaf")
        }
    }
}

struct App {
    shown: Shown,
}

impl eframe::App for App {
    // egui 0.36 hands the app a `Ui` rather than a `Context`, and that `Ui`
    // carries no margin or background of its own, so the panel is what gives
    // the window its own. Drawing lives in `render` so that a test can drive
    // it without a `Frame`, which belongs to the runner.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.render(ui);
    }
}

impl App {
    fn render(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::default().show(ui, |ui| match &mut self.shown {
            Shown::Nothing => {
                ui.label("Open a file: flyleaf path/to/file.toml");
            }
            Shown::Failed { path, why } => {
                ui.label(path.display().to_string());
                ui.add_space(8.0);
                ui.label(egui::RichText::new(why.as_str()).color(ui.visuals().error_fg_color));
            }
            Shown::Document { path, doc } => {
                ui.horizontal(|ui| {
                    ui.label(path.display().to_string());
                    ui.label(
                        egui::RichText::new("edits stay in memory; saving is not built yet")
                            .italics()
                            .weak(),
                    );
                });
                ui.add_space(8.0);
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| flyleaf::render(ui, doc, &()));
            }
        });
    }
}

fn main() -> eframe::Result {
    let shown = shown(std::env::args_os().nth(1).map(PathBuf::from));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title(&shown))
            .with_inner_size([900.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Tommy Flyleaf",
        options,
        Box::new(|_cc| Ok(Box::new(App { shown }))),
    )
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{open, shown, title, Shown};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden/fixtures")
            .join(name)
    }

    /// A file that opens is the document it holds, unchanged: this is the
    /// shell, and the shell must not touch what it shows.
    #[test]
    fn a_file_opens_as_itself() {
        let path = fixture("every-type.toml");
        let doc = open(&path).expect("the fixture opens");
        assert_eq!(doc.to_string(), std::fs::read_to_string(&path).unwrap());
    }

    /// A missing file and a file that is not TOML are both refusals with a
    /// reason, and the reason for the second names the line, because that
    /// is what the window shows and a bare "parse error" would send somebody
    /// to another tool to find out where.
    #[test]
    fn a_file_that_will_not_open_says_why() {
        let missing = open(Path::new("/nowhere/at/all.toml")).unwrap_err();
        assert!(!missing.is_empty());

        let dir = std::env::temp_dir().join(format!("flyleaf-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.toml");
        std::fs::write(&bad, "a = 1\nb = \n").unwrap();
        let why = open(&bad).unwrap_err();
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(why.contains("line 2"), "{why}");
    }

    /// No argument shows the window with nothing in it rather than failing,
    /// and a bad argument shows the window with the reason rather than
    /// exiting: on Windows this executable has no console to exit into.
    #[test]
    fn every_argument_produces_a_window() {
        assert!(matches!(shown(None), Shown::Nothing));
        assert!(matches!(
            shown(Some(PathBuf::from("/nowhere/at/all.toml"))),
            Shown::Failed { .. }
        ));
        assert!(matches!(
            shown(Some(fixture("every-type.toml"))),
            Shown::Document { .. }
        ));
    }

    /// Every state the window can be in draws without panicking, headlessly.
    /// The one with a document reaches the widget, which is the point of the
    /// shell.
    #[test]
    fn every_state_draws() {
        for shown in [
            shown(None),
            shown(Some(PathBuf::from("/nowhere/at/all.toml"))),
            shown(Some(fixture("every-type.toml"))),
        ] {
            let mut app = super::App { shown };
            egui::__run_test_ui(|ui| app.render(ui));
        }
    }

    /// The file's name leads the title, so a task bar reads as files.
    #[test]
    fn the_title_leads_with_the_file() {
        assert_eq!(title(&Shown::Nothing), "Tommy Flyleaf");
        let failed = Shown::Failed {
            path: PathBuf::from("/some/where/Cargo.toml"),
            why: String::new(),
        };
        assert_eq!(title(&failed), "Cargo.toml \u{2014} Tommy Flyleaf");
    }
}
