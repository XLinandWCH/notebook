#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use std::fs;
use std::path::PathBuf;

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 650.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("Notebook"),
        ..Default::default()
    };

    eframe::run_native(
        "Notebook",
        options,
        Box::new(|_cc| Ok(Box::new(NotebookApp::new()))),
    )
    .unwrap();
}

struct NotebookApp {
    content: String,
    file_path: Option<PathBuf>,
    is_modified: bool,
    word_wrap: bool,
    show_about: bool,
    cursor_line: usize,
    cursor_col: usize,
}

impl NotebookApp {
    fn new() -> Self {
        Self {
            content: String::new(),
            file_path: None,
            is_modified: false,
            word_wrap: false,
            show_about: false,
            cursor_line: 1,
            cursor_col: 1,
        }
    }

    fn title(&self) -> String {
        let name = match &self.file_path {
            Some(p) => p.file_name().unwrap().to_string_lossy().to_string(),
            None => "Untitled".to_string(),
        };
        let mark = if self.is_modified { " *" } else { "" };
        format!("{} - Notebook{}", name, mark)
    }

    fn update_cursor(&mut self) {
        let bytes = self.content.as_bytes();
        let pos = self.content.len().min(bytes.len());
        let mut line = 1;
        let mut col = 1;
        for &b in &bytes[..pos] {
            if b == b'\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        self.cursor_line = line;
        self.cursor_col = col;
    }

    fn do_new(&mut self) {
        if self.is_modified {
            // In a real app, prompt to save. For simplicity, just clear.
        }
        self.content.clear();
        self.file_path = None;
        self.is_modified = false;
        self.update_cursor();
    }

    fn do_open(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            match fs::read_to_string(&path) {
                Ok(text) => {
                    self.content = text;
                    self.file_path = Some(path);
                    self.is_modified = false;
                    self.update_cursor();
                }
                Err(e) => {
                    eprintln!("Failed to open file: {}", e);
                }
            }
        }
    }

    fn do_save(&mut self) -> bool {
        if let Some(path) = &self.file_path {
            match fs::write(path, &self.content) {
                Ok(_) => {
                    self.is_modified = false;
                    return true;
                }
                Err(e) => {
                    eprintln!("Failed to save file: {}", e);
                    return false;
                }
            }
        } else {
            self.do_save_as()
        }
    }

    fn do_save_as(&mut self) -> bool {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text Files", &["txt"])
            .add_filter("All Files", &["*"])
            .set_file_name("untitled.txt")
            .save_file()
        {
            match fs::write(&path, &self.content) {
                Ok(_) => {
                    self.file_path = Some(path);
                    self.is_modified = false;
                    return true;
                }
                Err(e) => {
                    eprintln!("Failed to save file: {}", e);
                    return false;
                }
            }
        }
        false
    }
}

impl eframe::App for NotebookApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_cursor();

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New          Ctrl+N").clicked() {
                    self.do_new();
                    ui.close_menu();
                }
                if ui.button("Open...      Ctrl+O").clicked() {
                    self.do_open();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Save         Ctrl+S").clicked() {
                    self.do_save();
                    ui.close_menu();
                }
                if ui.button("Save As...   Ctrl+Shift+S").clicked() {
                    self.do_save_as();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Exit         Alt+F4").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ui.menu_button("Edit", |ui| {
                if ui.button("Undo         Ctrl+Z").clicked() {
                    // eframe/egui TextEdit doesn't support undo natively in this setup
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Cut          Ctrl+X").clicked() {
                    ui.ctx().copy_to_clipboard(egui::ClipboardKind::Selection);
                    // We can't delete selected text easily without MultiEditor
                    ui.close_menu();
                }
                if ui.button("Copy         Ctrl+C").clicked() {
                    ui.ctx().copy_to_clipboard(egui::ClipboardKind::Selection);
                    ui.close_menu();
                }
                if ui.button("Paste        Ctrl+V").clicked() {
                    if let Some(s) = ui.ctx().clipboard_text() {
                        self.content.push_str(&s);
                        self.is_modified = true;
                    }
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Select All   Ctrl+A").clicked() {
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Find...      Ctrl+F").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("View", |ui| {
                let mut ww = self.word_wrap;
                if ui.checkbox(&mut ww, "Word Wrap").changed() {
                    self.word_wrap = ww;
                }
                if ui.button("Zoom In      Ctrl++").clicked() {
                    ui.close_menu();
                }
                if ui.button("Zoom Out     Ctrl+-").clicked() {
                    ui.close_menu();
                }
                if ui.button("Restore Zoom Ctrl+0").clicked() {
                    ui.close_menu();
                }
            });

            ui.menu_button("Help", |ui| {
                if ui.button("About Notebook").clicked() {
                    self.show_about = true;
                    ui.close_menu();
                }
            });
        });

        // Main text area
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut text_clone = self.content.clone();

            let text_edit = egui::TextEdit::multiline(&mut text_clone)
                .hint_text("Start typing...")
                .desired_width(f32::INFINITY)
                .desired_rows(20)
                .frame(false);

            let response = ui.add(text_edit);

            if text_clone != self.content {
                self.content = text_clone;
                self.is_modified = true;
            }

            if self.word_wrap {
                response.scroll_to_me_some(egui::Align::Center);
            }
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let path_text = self.file_path
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "Untitled".to_string());

                ui.label(format!("Ln {}, Col {}", self.cursor_line, self.cursor_col));
                ui.separator();
                ui.label(format!("{}", path_text));
                if self.is_modified {
                    ui.label(" [Modified]");
                }
            });
        });

        // About dialog
        if self.show_about {
            egui::Window::new("About Notebook")
                .open(&mut self.show_about)
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Notebook v0.1.0");
                    ui.label("");
                    ui.label("A simple text editor built with Rust and eframe/egui.");
                    ui.label("");
                    ui.label("Inspired by Windows 11 Notepad.");
                    ui.separator();
                    if ui.button("OK").clicked() {
                        self.show_about = false;
                    }
                });
        }

        // Keyboard shortcuts
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::N)) {
            self.do_new();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::O)) {
            self.do_open();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::S)) && !ctx.input(|i| i.modifiers.shift) {
            self.do_save();
        }
        if ctx.input(|i| i.modifiers.ctrl && i.modifiers.shift && i.key_pressed(egui::Key::S)) {
            self.do_save_as();
        }

        // Auto-save viewport title
        _frame.set_window_title(&self.title());
    }
}
