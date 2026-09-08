//! Tommy Flyleaf, the application: a window around the widget.
//!
//! `flyleaf path/to/file.toml` opens that file and shows it as a tree beside
//! what a save would write, with undo and redo, open, save and save-as, and a
//! prompt before unsaved changes are lost. One optional argument, no flags
//! and no subcommands; on Windows this is a GUI-subsystem executable and
//! prints nothing, which is why every error is shown in the window rather
//! than written anywhere.
//!
//! The same shell runs in a browser, where a file has no path: it arrives as
//! bytes from the browser's picker and leaves as a download, and a dialog
//! runs on a future rather than a thread, there being none. Those are the
//! only arms this file has, and each is marked with the target it is for.
//
// Author: David M. Anderson
// Built with AI assistance (Claude, Anthropic)

// `deny` rather than `forbid`, for one module: macOS delivers a double-clicked
// document as an Apple Event and never as an argument, and receiving one means
// defining an Objective-C class, which `objc2` cannot do without `unsafe`.
// `forbid` cannot be lifted beneath it, so the application crate carries
// `deny` and the handler carries the one `allow`; the widget in `lib.rs` and
// `flyleaf-core` stay `forbid`. `CLAUDE.md` records the decision.
#![deny(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![cfg_attr(windows, windows_subsystem = "windows")]

// The one exception to `deny(unsafe_code)` above. Nothing else in this crate
// is permitted it, and a second module wanting the allow is a decision.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod opened_document;

use std::ops::Range;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::sync::mpsc;

use flyleaf_core::Document;

/// The name the binary, the Linux desktop entry and the reverse-DNS bundle
/// identifier share. `with_app_id` is Wayland's `xdg_toplevel.set_app_id`,
/// which is how a compositor finds the window's icon; it does nothing on the
/// other two platforms, where the bundle and the executable's path do that.
#[cfg(not(target_arch = "wasm32"))]
const APP_ID: &str = "flyleaf";

/// The icon the window shows on Windows, as bytes, because a window icon
/// there comes from a resource compiled into the executable and a resource
/// compiler is what `CLAUDE.md` keeps out of the build. The 64-pixel entry
/// is decoded at startup; slipcase-desktop's README records why that size.
#[cfg(target_os = "windows")]
const WINDOW_ICON: &[u8] = include_bytes!("../../packaging/windows/flyleaf.ico");

#[cfg(target_os = "windows")]
fn window_icon() -> Option<egui::IconData> {
    let directory = ico::IconDir::read(std::io::Cursor::new(WINDOW_ICON)).ok()?;
    let entry = directory.entries().iter().find(|e| e.width() == 64)?;
    let image = entry.decode().ok()?;
    Some(egui::IconData {
        rgba: image.rgba_data().to_vec(),
        width: image.width(),
        height: image.height(),
    })
}

/// Where a document is. A path on disk; on the web a name, which is all a
/// browser will say about a file it hands over.
#[cfg(not(target_arch = "wasm32"))]
type Place = PathBuf;
#[cfg(target_arch = "wasm32")]
type Place = String;

/// What the window shows.
enum Shown {
    /// No argument was given, or nothing has been opened yet. The web build
    /// starts on the sample instead and never constructs this, and the
    /// variant stays so that the two builds match arms in one place.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    Nothing,
    /// The file, parsed. Boxed: a document carries its history, and the
    /// other two variants are a place and a string.
    Document { at: Place, doc: Box<Document> },
    /// The file could not be read or was not TOML, and this is what was said.
    Failed { at: Place, why: String },
}

/// What a file's bytes become: the document, or what was wrong with them.
///
/// `toml_edit`'s error carries the line and a caret under the column, which is
/// what somebody wants to see beside a file that would not open.
fn parsed(at: Place, bytes: Result<Vec<u8>, String>) -> Shown {
    match bytes.and_then(|b| Document::from_bytes(&b).map_err(|e| e.to_string())) {
        Ok(doc) => Shown::Document {
            at,
            doc: Box::new(doc),
        },
        Err(why) => Shown::Failed { at, why },
    }
}

/// What a path opens, or what the window shows without one.
#[cfg(not(target_arch = "wasm32"))]
fn shown(arg: Option<PathBuf>) -> Shown {
    match arg {
        None => Shown::Nothing,
        Some(path) => {
            let bytes = std::fs::read(&path).map_err(|e| e.to_string());
            parsed(path, bytes)
        }
    }
}

/// The window's title: the file's name first, so that a task bar full of
/// windows reads as files rather than as a row of the same application, and
/// a mark on it while there is something unsaved.
fn title(shown: &Shown) -> String {
    match shown {
        Shown::Nothing => "Tommy Flyleaf".to_owned(),
        Shown::Document { at, doc } => {
            let mark = if doc.edited() { "\u{2022} " } else { "" };
            format!("{mark}{} \u{2014} Tommy Flyleaf", name_of(at))
        }
        Shown::Failed { at, .. } => format!("{} \u{2014} Tommy Flyleaf", name_of(at)),
    }
}

/// The file's own name, without the directory.
#[cfg(not(target_arch = "wasm32"))]
fn name_of(at: &Place) -> String {
    at.file_name().map_or_else(
        || at.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    )
}
#[cfg(target_arch = "wasm32")]
fn name_of(at: &Place) -> String {
    at.clone()
}

/// The place as the bar shows it: the whole path, or the name.
#[cfg(not(target_arch = "wasm32"))]
fn shown_at(at: &Place) -> String {
    at.display().to_string()
}
#[cfg(target_arch = "wasm32")]
fn shown_at(at: &Place) -> String {
    at.clone()
}

/// Which dialog is up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ask {
    /// A file to open.
    Open,
    /// Where to save the document, under what name.
    SaveAs,
}

/// A dialog running elsewhere, and the channel its answer comes back on.
///
/// The dialog blocks what it runs on until it is closed, and the window has
/// to go on drawing while it is up, so it runs elsewhere and is polled once
/// a frame: on its own thread, or on the web on a future, there being no
/// thread.
struct Picking {
    what: Ask,
    answer: mpsc::Receiver<Answer>,
}

/// What a dialog came back with.
enum Answer {
    /// Closed without choosing.
    Nothing,
    /// A file to open: where it is, and on the web its bytes, since the
    /// browser hands those over with the name and nothing can read the
    /// place again.
    Open(Place, Option<Vec<u8>>),
    /// Where the document was saved: a path the document is then written
    /// to, or on the web a name it was already downloaded under.
    SavedAs(Place),
}

/// What to do once unsaved changes have been dealt with.
#[derive(Clone, PartialEq, Eq, Debug)]
enum Then {
    /// Close the window.
    Close,
    /// Open another file.
    Open,
    /// Show this file, which the platform handed over: on macOS a document
    /// double-clicked into a running window arrives this way.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Show(Place),
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
    /// Whether the About box is up.
    about: bool,
    /// The title as last sent to the window, so it is sent only on change.
    titled: String,
    /// The context, for the one place that has to start a dialog with no
    /// `Ui` in hand: a save on the web, which is a download and so a dialog.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    ctx: egui::Context,
}

impl App {
    fn new(shown: Shown, ctx: egui::Context) -> Self {
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
            about: false,
            titled: String::new(),
            ctx,
        }
    }

    fn edited(&self) -> bool {
        matches!(&self.shown, Shown::Document { doc, .. } if doc.edited())
    }

    /// Where the file that is open is, so a dialog can start beside it, and
    /// its name, offered where the question is what to call the one being
    /// written.
    fn current(&self) -> Option<Place> {
        match &self.shown {
            Shown::Document { at, .. } | Shown::Failed { at, .. } => Some(at.clone()),
            Shown::Nothing => None,
        }
    }

    /// Put a dialog up, unless one already is.
    #[cfg(not(target_arch = "wasm32"))]
    fn start_picking(&mut self, ctx: &egui::Context, what: Ask) {
        if self.picking.is_some() {
            return;
        }
        let (sender, answer) = mpsc::channel();
        let ctx = ctx.clone();
        let current = self.current();
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
                Ask::Open => dialog
                    .set_title("Open a TOML file")
                    .pick_file()
                    .map_or(Answer::Nothing, |p| Answer::Open(p, None)),
                Ask::SaveAs => {
                    if let Some(name) = current.as_ref().and_then(|p| p.file_name()) {
                        dialog = dialog.set_file_name(name.to_string_lossy());
                    }
                    dialog
                        .set_title("Save as")
                        .save_file()
                        .map_or(Answer::Nothing, Answer::SavedAs)
                }
            };
            let _ = sender.send(chosen);
            // Nothing has been touching the window while the dialog was up,
            // so it is asleep and has to be woken to notice the answer.
            ctx.request_repaint();
        });
        self.picking = Some(Picking { what, answer });
    }

    /// The same on the web: the browser's picker on a future, a file read
    /// as bytes, and a save as a download of what the document renders to
    /// at the moment the dialog goes up.
    #[cfg(target_arch = "wasm32")]
    fn start_picking(&mut self, ctx: &egui::Context, what: Ask) {
        if self.picking.is_some() {
            return;
        }
        let (sender, answer) = mpsc::channel();
        let ctx = ctx.clone();
        let current = self.current();
        let bytes = match (&what, &self.shown) {
            (Ask::SaveAs, Shown::Document { doc, .. }) => doc.render().into_bytes(),
            _ => Vec::new(),
        };
        wasm_bindgen_futures::spawn_local(async move {
            let dialog = rfd::AsyncFileDialog::new().add_filter("TOML", &["toml"]);
            let chosen = match what {
                Ask::Open => match dialog.pick_file().await {
                    Some(file) => Answer::Open(file.file_name(), Some(file.read().await)),
                    None => Answer::Nothing,
                },
                Ask::SaveAs => {
                    let dialog =
                        dialog.set_file_name(current.unwrap_or_else(|| "document.toml".to_owned()));
                    match dialog.save_file().await {
                        Some(file) if file.write(&bytes).await.is_ok() => {
                            Answer::SavedAs(file.file_name())
                        }
                        _ => Answer::Nothing,
                    }
                }
            };
            let _ = sender.send(chosen);
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
            Err(mpsc::TryRecvError::Disconnected) => Answer::Nothing,
            Ok(chosen) => chosen,
        };
        self.picking = None;
        debug_assert!(matches!(
            (&chosen, what),
            (Answer::Nothing, _)
                | (Answer::Open(..), Ask::Open)
                | (Answer::SavedAs(_), Ask::SaveAs)
        ));
        match chosen {
            Answer::Nothing => {}
            Answer::Open(at, bytes) => self.show(opened(at, bytes)),
            Answer::SavedAs(at) => self.saved_as(at),
        }
    }

    /// The document is at a new place: written there, or on the web already
    /// downloaded under that name.
    #[cfg(not(target_arch = "wasm32"))]
    fn saved_as(&mut self, at: Place) {
        if let Shown::Document { at: here, .. } = &mut self.shown {
            *here = at;
        }
        self.save();
    }
    #[cfg(target_arch = "wasm32")]
    fn saved_as(&mut self, at: Place) {
        if let Shown::Document { at: here, doc } = &mut self.shown {
            *here = at;
            doc.mark_saved();
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
    #[cfg(not(target_arch = "wasm32"))]
    fn save(&mut self) -> bool {
        let Shown::Document { at, doc } = &mut self.shown else {
            return false;
        };
        match doc.save_to(at) {
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

    /// On the web a save is a download, and a download is a dialog, so this
    /// is Save as with the name filled in. Returns true once the dialog is
    /// up: what follows the prompt's Save can go ahead, since nothing here
    /// can wait for a download to land.
    #[cfg(target_arch = "wasm32")]
    fn save(&mut self) -> bool {
        if !matches!(self.shown, Shown::Document { .. }) {
            return false;
        }
        let ctx = self.ctx.clone();
        self.start_picking(&ctx, Ask::SaveAs);
        true
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
            Then::Show(at) => self.show(opened(at, None)),
        }
    }

    /// The prompt: save, don't save, or stay.
    fn ask(&mut self, ctx: &egui::Context) {
        let Some(then) = self.asking.clone() else {
            return;
        };
        let name = match &self.shown {
            Shown::Document { at, .. } => name_of(at),
            _ => String::new(),
        };
        let mut answer = None;
        egui::Modal::new(egui::Id::new("unsaved")).show(ctx, |ui| {
            ui.set_width(360.0);
            ui.heading(format!("Save changes to {name}?"));
            ui.label(match then {
                Then::Close => "The window is closing. Unsaved changes will be lost.",
                Then::Open | Then::Show(_) => {
                    "Another file is opening. Unsaved changes will be lost."
                }
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
        // A document the platform handed over is opened the way one chosen in
        // the dialog is, unsaved changes asked about first. Not while a dialog
        // is up, since the answer to that dialog is a file too.
        #[cfg(target_os = "macos")]
        if self.picking.is_none() {
            if let Some(path) = opened_document::taken() {
                self.after_asking(ui.ctx(), Then::Show(path));
            }
        }

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
        self.about(ui.ctx());

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
            if let Shown::Document { at, doc } = &self.shown {
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
                ui.label(shown_at(at));
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
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("About").clicked() {
                    self.about = true;
                }
            });
        });
        pressed
    }

    /// The About box: the icon, the name and version, what the application
    /// is, where it comes from, and the name's story. Escape, the backdrop or
    /// the button closes it.
    fn about(&mut self, ctx: &egui::Context) {
        if !self.about {
            return;
        }
        let response = egui::Modal::new(egui::Id::new("about")).show(ctx, |ui| {
            ui.set_width(420.0);
            ui.horizontal(|ui| {
                icon(ui, 56.0);
                ui.vertical(|ui| {
                    ui.heading("Tommy Flyleaf");
                    ui.label(concat!("Version ", env!("CARGO_PKG_VERSION")));
                });
            });
            ui.add_space(8.0);
            ui.label(
                "A TOML editor that shows the file as a tree and edits every value by \
                 its kind, then saves a file with the comments, key order, whitespace \
                 and quoting untouched everywhere you did not edit.",
            );
            ui.add_space(6.0);
            ui.label(
                "Named for a bookbinder's apprentice. The flyleaf is the blank page \
                 inside a book's cover, the one place in a bound book meant for \
                 someone to write on later. The book is somebody else's; write on \
                 the page provided, and leave the rest as it was bound.",
            );
            ui.add_space(6.0);
            ui.hyperlink_to("excelano.com/flyleaf", "https://excelano.com/flyleaf/");
            ui.hyperlink_to(
                "github.com/excelano/flyleaf",
                "https://github.com/excelano/flyleaf",
            );
            ui.add_space(6.0);
            ui.label(
                egui::RichText::new(
                    "MIT licensed. Written by David M. Anderson, with AI assistance \
                     (Claude, Anthropic).",
                )
                .weak(),
            );
            ui.add_space(8.0);
            if ui.button("Close").clicked() {
                self.about = false;
            }
        });
        if response.should_close() {
            self.about = false;
        }
    }

    /// Below the bar: nothing, a refusal, or the pane and the tree.
    fn body(&mut self, ui: &mut egui::Ui, action: Option<Action>) {
        ui.add_space(8.0);
        let doc = match &mut self.shown {
            Shown::Nothing => {
                ui.label("Open a file, or start with one: flyleaf path/to/file.toml");
                return;
            }
            Shown::Failed { at, why } => {
                ui.label(shown_at(at));
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

/// The application icon, painted from the geometry of
/// `packaging/linux/icons/flyleaf.svg` rather than loaded from a raster, so
/// that the repository keeps one drawing and no bitmap. The two are kept in
/// step by hand: the SVG's comment names its shapes, and these are them,
/// on the same 64-unit grid, scaled to `size`.
fn icon(ui: &mut egui::Ui, size: f32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), egui::Sense::hover());
    let unit = size / 64.0;
    let at = |x: f32, y: f32| rect.min + egui::vec2(x * unit, y * unit);
    let outline = egui::Stroke::new(3.0 * unit, egui::Color32::from_rgb(0x2f, 0x33, 0x37));
    let oxblood = egui::Color32::from_rgb(0x5c, 0x22, 0x31);
    let brass = egui::Color32::from_rgb(0xc9, 0x96, 0x3a);
    let cream = egui::Color32::from_rgb(0xf2, 0xec, 0xe1);
    let fold = egui::Color32::from_rgb(0xd9, 0xcf, 0xbf);
    let painter = ui.painter();
    // The cover, then the spine over its left edge, then the page over both.
    let cover = egui::Rect::from_min_max(at(6.0, 4.0), at(58.0, 60.0));
    painter.rect(
        cover,
        3.0 * unit,
        oxblood,
        outline,
        egui::StrokeKind::Middle,
    );
    let spine = egui::Rect::from_min_max(at(6.0, 4.0), at(15.0, 60.0));
    // A corner radius is a u8 of pixels in egui, and the icon is drawn at
    // sizes under a few hundred pixels, so the radius is a small positive
    // number that fits; rounded first so that 2.6 becomes 3, not 2.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let rounding = (3.0 * unit).round().clamp(0.0, 255.0) as u8;
    painter.rect_filled(
        spine,
        egui::CornerRadius {
            nw: rounding,
            sw: rounding,
            ne: 0,
            se: 0,
        },
        brass,
    );
    painter.line_segment([at(15.0, 4.0), at(15.0, 60.0)], outline);
    painter.add(egui::Shape::convex_polygon(
        vec![
            at(20.0, 10.0),
            at(45.0, 10.0),
            at(53.0, 18.0),
            at(53.0, 54.0),
            at(20.0, 54.0),
        ],
        cream,
        outline,
    ));
    painter.add(egui::Shape::convex_polygon(
        vec![at(45.0, 10.0), at(45.0, 18.0), at(53.0, 18.0)],
        fold,
        outline,
    ));
    // Three lines of the tree: a key, and two beneath it, indented.
    let line = egui::Stroke::new(3.0 * unit, oxblood);
    painter.line_segment([at(27.0, 27.0), at(40.0, 27.0)], line);
    painter.line_segment([at(31.0, 36.0), at(46.0, 36.0)], line);
    painter.line_segment([at(31.0, 45.0), at(41.0, 45.0)], line);
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

/// What a dialog's answer opens.
#[cfg(not(target_arch = "wasm32"))]
fn opened(at: Place, _bytes: Option<Vec<u8>>) -> Shown {
    shown(Some(at))
}
#[cfg(target_arch = "wasm32")]
fn opened(at: Place, bytes: Option<Vec<u8>>) -> Shown {
    parsed(at, Ok(bytes.unwrap_or_default()))
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let shown = shown(std::env::args_os().nth(1).map(PathBuf::from));
    let viewport = egui::ViewportBuilder::default()
        .with_title(title(&shown))
        .with_app_id(APP_ID)
        .with_inner_size([1100.0, 700.0]);
    #[cfg(target_os = "windows")]
    let viewport = match window_icon() {
        Some(icon) => viewport.with_icon(icon),
        None => viewport,
    };
    // An empty `IconData` is how the icon is declined rather than replaced:
    // eframe substitutes its own egui logo for a viewport that names no icon
    // and hands it to `setApplicationIconImage:`, which outranks the bundle's
    // `.icns` in the Dock. Measured in slipcase-desktop and segler, both by a
    // person looking at the Dock.
    #[cfg(target_os = "macos")]
    let viewport = viewport.with_icon(egui::IconData::default());

    // Before `eframe` runs, since the handler has to be on at
    // `applicationWillFinishLaunching:`; `opened_document` says why.
    #[cfg(target_os = "macos")]
    opened_document::watch();

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Tommy Flyleaf",
        options,
        Box::new(|cc| {
            #[cfg(target_os = "macos")]
            opened_document::wake_with(&cc.egui_ctx);
            Ok(Box::new(App::new(shown, cc.egui_ctx.clone())))
        }),
    )
}

/// The same application in the page's canvas. A browser has no argument to
/// give, so the page opens on the sample file instead of on nothing: an empty
/// editor shows a visitor a button, and the sample shows them the editor.
/// Their own file comes through the picker, as before.
#[cfg(target_arch = "wasm32")]
fn main() {
    use web_sys::wasm_bindgen::JsCast as _;
    let sample = parsed(
        "sample.toml".to_owned(),
        Ok(include_bytes!("../../packaging/sample.toml").to_vec()),
    );
    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("flyleaf"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("the page has a canvas called flyleaf");
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(App::new(sample, cc.egui_ctx.clone())))),
            )
            .await
            .expect("the application starts");
    });
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{parsed, shown, title, App, Shown, Then};

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

    fn why(shown: &Shown) -> &str {
        match shown {
            Shown::Failed { why, .. } => why,
            _ => "",
        }
    }

    /// A file that opens is the document it holds, unchanged: this is the
    /// shell, and the shell must not touch what it shows.
    #[test]
    fn a_file_opens_as_itself() {
        let path = fixture("every-type.toml");
        let Shown::Document { doc, .. } = shown(Some(path.clone())) else {
            panic!("the fixture opens");
        };
        assert_eq!(doc.render(), std::fs::read_to_string(&path).unwrap());
    }

    /// A missing file and a file that is not TOML are both refusals with a
    /// reason, and the reason for the second names the line, because that
    /// is what the window shows and a bare "parse error" would send somebody
    /// to another tool to find out where.
    #[test]
    fn a_file_that_will_not_open_says_why() {
        let missing = shown(Some(PathBuf::from("/nowhere/at/all.toml")));
        assert!(!why(&missing).is_empty());

        let dir = std::env::temp_dir().join(format!("flyleaf-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bad = dir.join("bad.toml");
        std::fs::write(&bad, "a = 1\nb = \n").unwrap();
        let bad = shown(Some(bad));
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(why(&bad).contains("line 2"), "{}", why(&bad));
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
            let mut app = App::new(shown, egui::Context::default());
            egui::__run_test_ui(|ui| app.render(ui));
        }
    }

    /// Ctrl+Z undoes the last step and Ctrl+Shift+Z redoes it, through the
    /// window, with the chord consumed before a field could take it.
    #[test]
    fn the_undo_and_redo_chords_reach_the_document() {
        let mut app = App::new(
            shown(Some(fixture("every-type.toml"))),
            egui::Context::default(),
        );
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
        let mut app = App::new(shown(Some(path.clone())), egui::Context::default());
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
        let mut app = App::new(
            shown(Some(fixture("every-type.toml"))),
            egui::Context::default(),
        );
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
    /// The web build opens on `packaging/sample.toml`, and a sample that does
    /// not parse would open on its error message instead. The native suite is
    /// where that is caught, since the wasm build runs no tests.
    /// The About box is a modal, and a modal that Escape does not close is a
    /// window nobody can get out of without the mouse. The button is drawn
    /// too, but a click cannot be sent through a bare context, so this is the
    /// keyboard's half.
    #[test]
    fn escape_closes_the_about_box() {
        let mut app = App::new(Shown::Nothing, egui::Context::default());
        let ctx = egui::Context::default();
        app.about = true;
        frame(&ctx, &mut app, Vec::new(), false);
        assert!(app.about, "a frame with nothing pressed keeps it up");
        let escape = egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        frame(&ctx, &mut app, vec![escape], false);
        assert!(!app.about, "Escape did not close the About box");
    }

    #[test]
    fn the_sample_the_web_build_opens_with_parses() {
        let sample = parsed(
            PathBuf::from("sample.toml"),
            Ok(include_bytes!("../../packaging/sample.toml").to_vec()),
        );
        assert!(
            matches!(sample, Shown::Document { .. }),
            "the sample did not parse"
        );
    }

    #[test]
    fn the_title_leads_with_the_file() {
        assert_eq!(title(&Shown::Nothing), "Tommy Flyleaf");
        let failed = Shown::Failed {
            at: PathBuf::from("/some/where/Cargo.toml"),
            why: String::new(),
        };
        assert_eq!(title(&failed), "Cargo.toml \u{2014} Tommy Flyleaf");
    }
}
