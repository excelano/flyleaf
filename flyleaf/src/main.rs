//! Tommy Flyleaf, the application: a window around the widget.
//!
//! `flyleaf path/to/file.toml` opens that file and shows it as a tree beside
//! what a save would write, with undo and redo, open, save and save-as, and a
//! prompt before unsaved changes are lost. One optional argument, no flags
//! and no subcommands; on Windows this is a GUI-subsystem executable and
//! prints nothing, which is why every error is shown in the window rather
//! than written anywhere.
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::mpsc;

use flyleaf_core::Document;

/// What the window shows.
enum Shown {
    /// No argument was given, or nothing has been opened yet.
    Nothing,
    /// The file, parsed. Boxed: a document carries its history, and the
    /// other two variants are a path and a string.
    Document { path: PathBuf, doc: Box<Document> },
    /// The file could not be read or was not TOML, and this is what was said.
    Failed { path: PathBuf, why: String },
}

/// Read and parse a file, with the reason where either fails.
///
/// `toml_edit`'s error carries the line and a caret under the column, which is
/// what somebody wants to see beside a file that would not open.
fn open(path: &Path) -> Result<Document, String> {
    Document::from_path(path).map_err(|e| e.to_string())
}

/// What a path opens, or what the window shows without one.
fn shown(arg: Option<PathBuf>) -> Shown {
    match arg {
        None => Shown::Nothing,
        Some(path) => match open(&path) {
            Ok(doc) => Shown::Document {
                path,
                doc: Box::new(doc),
            },
            Err(why) => Shown::Failed { path, why },
        },
    }
}

/// The window's title: the file's name first, so that a task bar full of
/// windows reads as files rather than as a row of the same application, and
/// a mark on it while there is something unsaved.
fn title(shown: &Shown) -> String {
    match shown {
        Shown::Nothing => "Tommy Flyleaf".to_owned(),
        Shown::Document { path, doc } => {
            let mark = if doc.edited() { "\u{2022} " } else { "" };
            format!("{mark}{} \u{2014} Tommy Flyleaf", name_of(path))
        }
        Shown::Failed { path, .. } => format!("{} \u{2014} Tommy Flyleaf", name_of(path)),
    }
}

fn name_of(path: &Path) -> String {
    path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}

/// Which dialog is up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ask {
    /// A file to open.
    Open,
    /// Where to save the document, under what name.
    SaveAs,
}

/// A dialog on its own thread, and the channel its answer comes back on.
///
/// The dialog blocks the thread it runs on until it is closed, and the
/// window has to go on drawing while it is up, so it runs elsewhere and is
/// polled once a frame.
struct Picking {
    what: Ask,
    answer: mpsc::Receiver<Option<PathBuf>>,
}

/// What to do once unsaved changes have been dealt with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Then {
    /// Close the window.
    Close,
    /// Open another file.
    Open,
}

struct App {
    shown: Shown,
    /// The row being worked in, as the tree reported it last frame; drawn
    /// above the tree, which is why it is a frame behind.
    selected: Option<Vec<String>>,
    /// Whether the source pane is shown. On by default: what a save would
    /// write is half of what this editor is for.
    show_source: bool,
    /// The lines of the source the selected row occupies, found again only
    /// when the selection or the document changes, since finding them is a
    /// parse.
    highlight: Option<Range<usize>>,
    /// The selection the highlight was found for.
    highlighted: Option<Vec<String>>,
    /// The dialog that is up, if one is.
    picking: Option<Picking>,
    /// What the last save said, where it failed. Shown until the next one.
    said: Option<String>,
    /// The prompt about unsaved changes, and what follows answering it.
    asking: Option<Then>,
    /// Whether a close may go ahead: set once the prompt has been answered,
    /// or where there was nothing to ask about.
    may_close: bool,
    /// The title as last sent to the window, so it is sent only on change.
    titled: String,
}

impl App {
    fn new(shown: Shown) -> Self {
        Self {
            shown,
            selected: None,
            show_source: true,
            highlight: None,
            highlighted: None,
            picking: None,
            said: None,
            asking: None,
            may_close: false,
            titled: String::new(),
        }
    }

    fn edited(&self) -> bool {
        matches!(&self.shown, Shown::Document { doc, .. } if doc.edited())
    }

    /// Put a dialog up, unless one already is.
    fn start_picking(&mut self, ctx: &egui::Context, what: Ask) {
        if self.picking.is_some() {
            return;
        }
        let (sender, answer) = mpsc::channel();
        let ctx = ctx.clone();
        // Beside the file that is open, under its name where the question
        // is what to call the one being written.
        let current = match &self.shown {
            Shown::Document { path, .. } | Shown::Failed { path, .. } => Some(path.clone()),
            Shown::Nothing => None,
        };
        std::thread::spawn(move || {
            let mut dialog = rfd::FileDialog::new()
                .add_filter("TOML", &["toml"])
                .add_filter("All files", &["*"]);
            if let Some(dir) = current.as_ref().and_then(|p| p.parent()) {
                if !dir.as_os_str().is_empty() {
                    dialog = dialog.set_directory(dir);
                }
            }
            let chosen = match what {
                Ask::Open => dialog.set_title("Open a TOML file").pick_file(),
                Ask::SaveAs => {
                    if let Some(name) = current.as_ref().and_then(|p| p.file_name()) {
                        dialog = dialog.set_file_name(name.to_string_lossy());
                    }
                    dialog.set_title("Save as").save_file()
                }
            };
            let _ = sender.send(chosen);
            // Nothing has been touching the window while the dialog was up,
            // so it is asleep and has to be woken to notice the answer.
            ctx.request_repaint();
        });
        self.picking = Some(Picking { what, answer });
    }

    /// Take the dialog's answer, once it has one.
    fn poll_picking(&mut self) {
        let Some(picking) = &self.picking else {
            return;
        };
        let what = picking.what;
        let chosen = match picking.answer.try_recv() {
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => None,
            Ok(chosen) => chosen,
        };
        self.picking = None;
        let Some(path) = chosen else {
            return;
        };
        match what {
            Ask::Open => self.show(shown(Some(path))),
            Ask::SaveAs => {
                if let Shown::Document { path: at, .. } = &mut self.shown {
                    *at = path;
                }
                self.save();
            }
        }
    }

    /// Show something else, forgetting the selection and any typing that
    /// belonged to what was shown before.
    fn show(&mut self, shown: Shown) {
        self.shown = shown;
        self.selected = None;
        self.highlight = None;
        self.highlighted = None;
        self.said = None;
        self.may_close = false;
    }

    /// Write the document to its path, and say so where it fails. Returns
    /// whether it was written.
    fn save(&mut self) -> bool {
        let Shown::Document { path, doc } = &mut self.shown else {
            return false;
        };
        match doc.save_to(path) {
            Ok(()) => {
                self.said = None;
                true
            }
            Err(e) => {
                self.said = Some(format!("not saved: {e}"));
                false
            }
        }
    }

    /// Ask about unsaved changes before doing something that would lose
    /// them, or just do it where there are none.
    fn after_asking(&mut self, ctx: &egui::Context, then: Then) {
        if self.edited() {
            self.asking = Some(then);
        } else {
            self.proceed(ctx, then);
        }
    }

    fn proceed(&mut self, ctx: &egui::Context, then: Then) {
        self.asking = None;
        match then {
            Then::Close => {
                self.may_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            Then::Open => self.start_picking(ctx, Ask::Open),
        }
    }

    /// The prompt: save, don't save, or stay.
    fn ask(&mut self, ctx: &egui::Context) {
        let Some(then) = self.asking else {
            return;
        };
        let name = match &self.shown {
            Shown::Document { path, .. } => name_of(path),
            _ => String::new(),
        };
        let mut answer = None;
        egui::Modal::new(egui::Id::new("unsaved")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading(format!("Save changes to {name}?"));
            ui.label(match then {
                Then::Close => "The window is closing. Unsaved changes will be lost.",
                Then::Open => "Another file is opening. Unsaved changes will be lost.",
            });
            if let Some(said) = &self.said {
                ui.label(egui::RichText::new(said.as_str()).color(ui.visuals().error_fg_color));
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    answer = Some(Some(true));
                }
                if ui.button("Don't save").clicked() {
                    answer = Some(Some(false));
                }
                if ui.button("Cancel").clicked() {
                    answer = Some(None);
                }
            });
        });
        // A save that fails keeps the prompt up with the reason in it, so
        // that nothing is lost on the strength of a click.
        let go_on = match answer {
            None => return,
            Some(None) => {
                self.asking = None;
                false
            }
            Some(Some(save)) => !save || self.save(),
        };
        if go_on {
            self.proceed(ctx, then);
        }
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        self.poll_picking();

        // Closing is the window's request; with something unsaved it is
        // refused this frame and asked about, and granted once answered.
        if ui.ctx().input(|i| i.viewport().close_requested()) && !self.may_close && self.edited() {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.asking = Some(Then::Close);
        }

        let mut action = chord(ui);
        let busy = self.picking.is_some() || self.asking.is_some();
        egui::CentralPanel::default().show(ui, |ui| {
            if let Some(pressed) = self.bar(ui, busy) {
                action = Some(pressed);
            }
            self.body(ui, action);
        });
        if !busy {
            match action {
                Some(Action::Open) => self.after_asking(ui.ctx(), Then::Open),
                Some(Action::SaveAs) => self.start_picking(ui.ctx(), Ask::SaveAs),
                Some(Action::Save) => {
                    self.save();
                }
                Some(Action::Undo | Action::Redo) | None => {}
            }
        }
        self.ask(ui.ctx());

        let title = title(&self.shown);
        if title != self.titled {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.titled = title;
        }
    }

    /// The bar across the top: the file, the actions, and what is going
    /// on. Returns the action a button asked for.
    fn bar(&mut self, ui: &mut egui::Ui, busy: bool) -> Option<Action> {
        let mut pressed = None;
        let mut press = |ui: &mut egui::Ui, enabled: bool, label: &str, action: Action| {
            if ui
                .add_enabled(enabled, egui::Button::new(label).small())
                .clicked()
            {
                pressed = Some(action);
            }
        };
        ui.horizontal(|ui| {
            press(ui, !busy, "Open\u{2026}", Action::Open);
            if let Shown::Document { path, doc } = &self.shown {
                press(ui, !busy && doc.edited(), "Save", Action::Save);
                press(ui, !busy, "Save as\u{2026}", Action::SaveAs);
                ui.separator();
                press(ui, doc.can_undo(), "Undo", Action::Undo);
                press(ui, doc.can_redo(), "Redo", Action::Redo);
                if ui.small_button("Expand all").clicked() {
                    flyleaf::open_all(ui.ctx(), true);
                }
                if ui.small_button("Collapse all").clicked() {
                    flyleaf::open_all(ui.ctx(), false);
                }
                ui.toggle_value(&mut self.show_source, "Source");
                ui.separator();
                ui.label(path.display().to_string());
                if let Some(selected) = &self.selected {
                    ui.label(egui::RichText::new(selected.join(".")).monospace().weak());
                }
                if doc.edited() {
                    ui.label(egui::RichText::new("edited").italics().weak());
                }
            }
            if let Some(said) = &self.said {
                ui.label(egui::RichText::new(said.as_str()).color(ui.visuals().error_fg_color));
            }
        });
        pressed
    }

    /// Below the bar: nothing, a refusal, or the pane and the tree.
    fn body(&mut self, ui: &mut egui::Ui, action: Option<Action>) {
        ui.add_space(8.0);
        let doc = match &mut self.shown {
            Shown::Nothing => {
                ui.label("Open a file, or start with one: flyleaf path/to/file.toml");
                return;
            }
            Shown::Failed { path, why } => {
                ui.label(path.display().to_string());
                ui.label(egui::RichText::new(why.as_str()).color(ui.visuals().error_fg_color));
                return;
            }
            Shown::Document { doc, .. } => doc,
        };

        let mut changed = false;
        match action {
            Some(Action::Undo) => {
                flyleaf::forget_typing(ui.ctx());
                changed = doc.undo();
            }
            Some(Action::Redo) => {
                flyleaf::forget_typing(ui.ctx());
                changed = doc.redo();
            }
            _ => {}
        }

        // The pane first, so that the tree gets what is left.
        if self.show_source {
            egui::Panel::right("source")
                .resizable(true)
                .default_size(ui.available_width() * 0.45)
                .show(ui, |ui| {
                    flyleaf::source(ui, &doc.render(), self.highlight.as_ref());
                });
        }
        self.selected = egui::ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| flyleaf::render(ui, doc.tree_mut(), &()))
            .inner;
        // Whatever this frame changed is a step, joined to the last one
        // where the same row is still being worked in.
        changed |= doc.record(self.selected.as_deref());

        if changed || self.selected != self.highlighted {
            self.highlight = self.selected.as_deref().and_then(|path| doc.lines_of(path));
            self.highlighted.clone_from(&self.selected);
        }
    }
}

/// What a chord or a button asks for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    Open,
    Save,
    SaveAs,
    Undo,
    Redo,
}

/// The chord pressed this frame, taken out of the input before the tree
/// draws so that a field with focus does not answer it with its own
/// reading. The shifted chords are asked first, since they contain the
/// plain ones.
fn chord(ui: &mut egui::Ui) -> Option<Action> {
    ui.input_mut(|i| {
        let cmd = egui::Modifiers::COMMAND;
        let shifted = cmd | egui::Modifiers::SHIFT;
        if i.consume_key(shifted, egui::Key::Z) || i.consume_key(cmd, egui::Key::Y) {
            Some(Action::Redo)
        } else if i.consume_key(cmd, egui::Key::Z) {
            Some(Action::Undo)
        } else if i.consume_key(shifted, egui::Key::S) {
            Some(Action::SaveAs)
        } else if i.consume_key(cmd, egui::Key::S) {
            Some(Action::Save)
        } else if i.consume_key(cmd, egui::Key::O) {
            Some(Action::Open)
        } else {
            None
        }
    })
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

fn main() -> eframe::Result {
    let shown = shown(std::env::args_os().nth(1).map(PathBuf::from));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title(&shown))
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Tommy Flyleaf",
        options,
        Box::new(|_cc| Ok(Box::new(App::new(shown)))),
    )
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{open, shown, title, App, Shown, Then};

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden/fixtures")
            .join(name)
    }

    /// A copy of a fixture in a directory of its own, for a test that saves.
    fn scratch(name: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "flyleaf-{}-{}",
            std::process::id(),
            name.replace('.', "-")
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::copy(fixture(name), &path).unwrap();
        (dir, path)
    }

    /// One frame of the window, with these events and this viewport state,
    /// returning the commands the window sent the viewport.
    fn frame(
        ctx: &egui::Context,
        app: &mut App,
        events: Vec<egui::Event>,
        close_requested: bool,
    ) -> Vec<egui::ViewportCommand> {
        let mut input = egui::RawInput {
            events,
            ..Default::default()
        };
        if close_requested {
            input
                .viewports
                .entry(egui::ViewportId::ROOT)
                .or_default()
                .events
                .push(egui::ViewportEvent::Close);
        }
        let output = ctx.run_ui(input, |ui| app.render(ui));
        let commands = output
            .viewport_output
            .get(&egui::ViewportId::ROOT)
            .map(|v| v.commands.clone())
            .unwrap_or_default();
        output.drop_without_applying_deltas();
        commands
    }

    fn chord(modifiers: egui::Modifiers, key: egui::Key) -> Vec<egui::Event> {
        vec![egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }]
    }

    /// A file that opens is the document it holds, unchanged: this is the
    /// shell, and the shell must not touch what it shows.
    #[test]
    fn a_file_opens_as_itself() {
        let path = fixture("every-type.toml");
        let doc = open(&path).expect("the fixture opens");
        assert_eq!(doc.render(), std::fs::read_to_string(&path).unwrap());
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
            let mut app = App::new(shown);
            egui::__run_test_ui(|ui| app.render(ui));
        }
    }

    /// Ctrl+Z undoes the last step and Ctrl+Shift+Z redoes it, through the
    /// window, with the chord consumed before a field could take it.
    #[test]
    fn the_undo_and_redo_chords_reach_the_document() {
        let mut app = App::new(shown(Some(fixture("every-type.toml"))));
        let Shown::Document { doc, .. } = &mut app.shown else {
            panic!("the fixture opens");
        };
        doc.tree_mut()["types"]["count"] = flyleaf_core::toml_edit::value(45);
        doc.record(None);
        let count = |app: &App| match &app.shown {
            Shown::Document { doc, .. } => doc.tree()["types"]["count"].as_integer(),
            _ => None,
        };

        let ctx = egui::Context::default();
        frame(
            &ctx,
            &mut app,
            chord(egui::Modifiers::COMMAND, egui::Key::Z),
            false,
        );
        assert_eq!(count(&app), Some(44), "undone");
        frame(
            &ctx,
            &mut app,
            chord(
                egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
                egui::Key::Z,
            ),
            false,
        );
        assert_eq!(count(&app), Some(45), "redone");
    }

    /// Ctrl+S writes the file and clears the edited mark, and the title
    /// carries the mark while there is something to save.
    #[test]
    fn the_save_chord_writes_the_file() {
        let (dir, path) = scratch("top-level-keys.toml");
        let mut app = App::new(shown(Some(path.clone())));
        let Shown::Document { doc, .. } = &mut app.shown else {
            panic!("the copy opens");
        };
        doc.tree_mut()["title"] = flyleaf_core::toml_edit::value("Q4 report");
        doc.record(None);
        assert!(
            title(&app.shown).starts_with('\u{2022}'),
            "marked in the title"
        );

        let ctx = egui::Context::default();
        frame(
            &ctx,
            &mut app,
            chord(egui::Modifiers::COMMAND, egui::Key::S),
            false,
        );
        assert!(!app.edited(), "saved");
        assert!(!title(&app.shown).starts_with('\u{2022}'));
        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.contains("title = \"Q4 report\""), "{written}");
        assert!(
            written.contains("author = \"D. Anderson\""),
            "the rest is as it was"
        );
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A close with unsaved changes is refused and asked about; with none it
    /// goes ahead. Without the refusal the window would close on the
    /// strength of a click on its corner and take the changes with it.
    #[test]
    fn a_close_with_unsaved_changes_is_refused_and_asked_about() {
        let mut app = App::new(shown(Some(fixture("every-type.toml"))));
        let ctx = egui::Context::default();
        let commands = frame(&ctx, &mut app, Vec::new(), true);
        assert!(
            !commands.contains(&egui::ViewportCommand::CancelClose),
            "nothing to ask about"
        );

        let Shown::Document { doc, .. } = &mut app.shown else {
            panic!("the fixture opens");
        };
        doc.tree_mut()["types"]["count"] = flyleaf_core::toml_edit::value(45);
        doc.record(None);
        let commands = frame(&ctx, &mut app, Vec::new(), true);
        assert!(commands.contains(&egui::ViewportCommand::CancelClose));
        assert_eq!(app.asking, Some(Then::Close));

        // Answered "don't save": the close is asked for and allowed.
        app.proceed(&ctx, Then::Close);
        assert!(app.may_close);
        let commands = frame(&ctx, &mut app, Vec::new(), true);
        assert!(!commands.contains(&egui::ViewportCommand::CancelClose));
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
