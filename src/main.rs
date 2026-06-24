use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui::backend::CrosstermBackend;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use std::{env, thread};

struct Editor {
    lines: Vec<String>,
    cursor_x: usize,
    cursor_y: usize,
    scroll_y: usize,
    file_path: Option<PathBuf>,
    modified: bool,
    tab_width: usize,
    wrap: bool,
    show_about: bool,
    quit: bool,
}

impl Editor {
    fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        let (lines, path) = if args.len() > 1 {
            let p = PathBuf::from(&args[1]);
            let text = std::fs::read_to_string(&p).unwrap_or_default();
            let lines = if text.is_empty() {
                vec![String::new()]
            } else {
                text.lines().map(|s| s.to_string()).collect()
            };
            (lines, Some(p))
        } else {
            (vec![String::new()], None)
        };
        Self {
            lines,
            cursor_x: 0,
            cursor_y: 0,
            scroll_y: 0,
            file_path: path,
            modified: false,
            tab_width: 4,
            wrap: false,
            show_about: false,
            quit: false,
        }
    }

    fn title(&self) -> String {
        let name = self.file_path
            .as_ref()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .unwrap_or_else(|| "Untitled".to_string());
        format!("{}{} - Notebook", name, if self.modified { " *" } else { "" })
    }

    fn total(&self) -> usize { self.lines.len() }

    fn insert_char(&mut self, c: char) {
        if self.cursor_y >= self.lines.len() {
            self.lines.push(String::new());
        }
        let line = &mut self.lines[self.cursor_y];
        let mut buf = [0u8; 4];
        line.insert_str(self.cursor_x, c.encode_utf8(&mut buf));
        self.cursor_x += 1;
        self.modified = true;
    }

    fn insert_tab(&mut self) {
        for _ in 0..self.tab_width { self.insert_char(' '); }
    }

    fn backspace(&mut self) {
        if self.cursor_x > 0 {
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                line.remove(self.cursor_x - 1);
            }
            self.cursor_x -= 1;
            self.modified = true;
        } else if self.cursor_y > 0 {
            let removed = self.lines.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0);
            if let Some(prev) = self.lines.get_mut(self.cursor_y) {
                prev.push_str(&removed);
            }
            self.modified = true;
        }
    }

    fn delete_char(&mut self) {
        if self.cursor_x < self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0) {
            if let Some(line) = self.lines.get_mut(self.cursor_y) {
                line.remove(self.cursor_x);
            }
            self.modified = true;
        } else if self.cursor_y + 1 < self.lines.len() {
            let next = self.lines.remove(self.cursor_y + 1);
            if let Some(curr) = self.lines.get_mut(self.cursor_y) {
                curr.push_str(&next);
            }
            self.modified = true;
        }
    }

    fn newline(&mut self) {
        let curr = self.lines.get(self.cursor_y).cloned().unwrap_or_default();
        let head: String = curr.chars().take(self.cursor_x).collect();
        let tail: String = curr.chars().skip(self.cursor_x).collect();
        if let Some(line) = self.lines.get_mut(self.cursor_y) {
            *line = head;
        }
        self.lines.insert(self.cursor_y + 1, tail);
        self.cursor_y += 1;
        self.cursor_x = 0;
        self.modified = true;
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0);
        }
    }

    fn move_right(&mut self) {
        let len = self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0);
        if self.cursor_x < len {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            let len = self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0);
            self.cursor_x = self.cursor_x.min(len);
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y + 1 < self.lines.len() {
            self.cursor_y += 1;
            let len = self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0);
            self.cursor_x = self.cursor_x.min(len);
        }
    }

    fn scroll_to_view(&mut self, vis_h: usize) {
        if self.cursor_y < self.scroll_y {
            self.scroll_y = self.cursor_y;
        } else if self.cursor_y >= self.scroll_y + vis_h {
            self.scroll_y = self.cursor_y - vis_h + 1;
        }
    }

    fn save(&mut self) -> io::Result<()> {
        let path = self.file_path.take().unwrap_or_else(|| {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join("untitled.txt")
        });
        let text = self.lines.join("\n");
        std::fs::write(&path, text)?;
        self.file_path = Some(path);
        self.modified = false;
        Ok(())
    }

    fn home(&mut self) { self.cursor_x = 0; }

    fn end(&mut self) {
        self.cursor_x = self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0);
    }

    fn doc_home(&mut self) {
        self.cursor_y = 0;
        self.cursor_x = 0;
        self.scroll_y = 0;
    }

    fn doc_end(&mut self) {
        self.cursor_y = self.lines.len().saturating_sub(1);
        self.cursor_x = self.lines.get(self.cursor_y).map(|s| s.len()).unwrap_or(0);
    }
}

fn build_text(editor: &Editor, width: usize, _height: usize) -> Text<'_> {
    let digits = (editor.total().max(1)).to_string().len().max(3);
    let mut lines = Vec::new();

    for y in 0..editor.lines.len() {
        let line_str = &editor.lines[y];
        let num_str = format!("{:>width$}", y + 1, width = digits);
        let is_cursor = y == editor.cursor_y;

        let num_style = if is_cursor {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)
        };

        let mut spans = vec![
            Span::styled(num_str.clone(), num_style),
            Span::raw(" "),
        ];

        for (i, c) in line_str.chars().enumerate() {
            let is_cur = is_cursor && i == editor.cursor_x;
            let (r, g, b) = token_color(c, i, line_str);
            let fg = if is_cur { Color::Black } else { Color::Rgb(r, g, b) };
            let bg = if is_cur { Color::Cyan } else { Color::Reset };
            spans.push(Span::styled(String::from(c), Style::default().fg(fg).bg(bg)));
        }

        if is_cursor && editor.cursor_x >= line_str.len() {
            spans.push(Span::styled(" ", Style::default().bg(Color::Cyan).fg(Color::Black)));
        }

        lines.push(Line::from(spans));
    }

    if lines.is_empty() {
        let digits = 3;
        let num_str = format!("{:>width$}", 1, width = digits);
        let num_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        lines.push(Line::from(vec![
            Span::styled(num_str.clone(), num_style),
            Span::raw(" "),
            Span::styled(" ", Style::default().bg(Color::Cyan).fg(Color::Black)),
        ]));
    }

    Text { lines, style: Style::default(), alignment: None }
}

fn token_color(c: char, _pos: usize, line: &str) -> (u8, u8, u8) {
    if c == '/' && line.chars().nth(_pos + 1) == Some('/') {
        (105, 113, 119)
    } else if c == '#' {
        (105, 113, 119)
    } else if c.is_ascii_digit() {
        (209, 142, 85)
    } else if c == '"' {
        (113, 188, 131)
    } else {
        (220, 220, 220)
    }
}

fn build_line_spans(line: &str) -> Vec<Span<'_>> {
    let mut spans = vec![];
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            spans.push(Span::raw(&line[i..]));
            break;
        } else if b == b'#' {
            spans.push(Span::raw(&line[i..]));
            break;
        } else if b == b'"' {
            let mut j = i + 1;
            while j < bytes.len() && !(bytes[j] == b'"' && bytes[j.saturating_sub(1)] != b'\\') {
                j += 1;
            }
            if j < bytes.len() { j += 1; }
            spans.push(Span::styled(&line[i..j], Style::default().fg(Color::Rgb(113, 188, 131))));
            i = j;
        } else if b.is_ascii_digit() {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
                j += 1;
            }
            spans.push(Span::styled(&line[i..j], Style::default().fg(Color::Rgb(209, 142, 85))));
            i = j;
        } else if b.is_ascii_alphabetic() || b == b'_' {
            let mut j = i;
            while j < bytes.len() && (bytes[j].is_ascii_alphanumeric() || bytes[j] == b'_') {
                j += 1;
            }
            let word = &line[i..j];
            let color = match word {
                "fn" | "let" | "mut" | "const" | "static" | "struct" | "enum" | "impl" | "trait"
                | "pub" | "mod" | "use" | "crate" | "self" | "super" | "match" | "if" | "else"
                | "for" | "while" | "loop" | "break" | "continue" | "return" | "in" | "as"
                | "where" | "type" | "async" | "await" | "move" | "ref" => Color::Rgb(205, 141, 206),
                "true" | "false" | "None" | "Some" => Color::Rgb(209, 142, 85),
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64"
                | "u128" | "usize" | "f32" | "f64" | "bool" | "char" | "str" | "String"
                | "Vec" | "Option" | "Result" | "Box" => Color::Rgb(144, 196, 234),
                _ => Color::Rgb(220, 220, 220),
            };
            spans.push(Span::styled(word, Style::default().fg(color)));
            i = j;
        } else {
            spans.push(Span::raw(String::from(bytes[i] as char)));
            i += 1;
        }
    }
    spans
}

fn render(editor: &Editor, f: &mut Frame) {
    let size = f.area();

    if editor.show_about {
        let area = Rect::new(
            size.width.saturating_sub(40) / 2,
            (size.height.saturating_sub(10)) / 2,
            40,
            10,
        );
        let block = Paragraph::new(
            "Notebook v0.1.0\n\nA simple text editor built with Rust.\nInspired by Windows 11 Notepad.\n\nPress any key to close..."
        )
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" About ")
                .border_style(Style::default().fg(Color::Cyan)))
            .alignment(Alignment::Center);
        f.render_widget(block, area);
        return;
    }

    let digits = (editor.total().max(1)).to_string().len().max(3);
    let gutter_w = (digits + 1) as u16;
    let edit_h = size.height.saturating_sub(2);

    // Menu bar
    let menus = ["File", "Edit", "View"];
    let menu_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Length(6); 3])
        .split(Rect::new(0, 0, size.width, 1));

    for (i, area) in menu_areas.iter().enumerate() {
        let label = menus[i];
        let style = if label == "File" && editor.modified {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        };
        let p = Paragraph::new(label).style(style);
        f.render_widget(p, *area);
    }

    // Editor
    let edit_area = Rect::new(0, 1, size.width, edit_h);

    let all_text = build_text(editor, size.width as usize, edit_h as usize);
    let edit_block = Paragraph::new(all_text)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: false });

    f.render_widget(edit_block, edit_area);

    // Status bar
    let status_area = Rect::new(0, size.height - 1, size.width, 1);
    let path_str = editor.file_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| "Untitled".to_string());
    let lc_str = format!("Ln {}, Col {}", editor.cursor_y + 1, editor.cursor_x + 1);
    let wrap_str = if editor.wrap { "Wrap" } else { "NoWrap" };

    let status_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(gutter_w),
            Constraint::Min(1),
            Constraint::Length((lc_str.len() + 1) as u16),
            Constraint::Length((wrap_str.len() + 1) as u16),
        ])
        .split(status_area);

    f.render_widget(Paragraph::new(path_str.as_str()).style(Style::default().fg(Color::Green)), status_layout[1]);
    f.render_widget(Paragraph::new(lc_str.as_str()).style(Style::default().fg(Color::Cyan)), status_layout[2]);
    f.render_widget(Paragraph::new(wrap_str).style(Style::default().fg(Color::DarkGray)), status_layout[3]);
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, SetTitle("Notebook"))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut editor = Editor::new();
    let (tx, rx) = mpsc::channel::<Event>();

    let tx2 = tx.clone();
    thread::spawn(move || {
        loop {
            if let Ok(evt) = event::read() {
                if tx2.send(evt).is_err() { break; }
            }
        }
    });

    loop {
        if editor.quit { break; }

        terminal.draw(|f| render(&editor, f))?;
        let size = terminal.size()?;
        let edit_h = (size.height.saturating_sub(2)) as usize;

        if let Ok(Event::Key(key)) = rx.recv_timeout(Duration::from_millis(50)) {
            if editor.show_about {
                editor.show_about = false;
                continue;
            }

            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            if ctrl {
                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        let _ = editor.save();
                        let _ = execute!(terminal.backend_mut(), SetTitle(&editor.title()));
                        continue;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        editor.lines = vec![String::new()];
                        editor.file_path = None;
                        editor.modified = false;
                        editor.cursor_x = 0;
                        editor.cursor_y = 0;
                        editor.scroll_y = 0;
                        let _ = execute!(terminal.backend_mut(), SetTitle(&editor.title()));
                        continue;
                    }
                    KeyCode::Char('q') | KeyCode::Char('Q') => { break; }
                    _ => {}
                }
            }

            match key.code {
                KeyCode::Esc => { break; }
                KeyCode::Left => { editor.move_left(); }
                KeyCode::Right => { editor.move_right(); }
                KeyCode::Up => { editor.move_up(); }
                KeyCode::Down => { editor.move_down(); }
                KeyCode::Home => { editor.home(); }
                KeyCode::End => { editor.end(); }
                KeyCode::PageUp => { for _ in 0..20 { editor.move_up(); } }
                KeyCode::PageDown => { for _ in 0..20 { editor.move_down(); } }
                KeyCode::Backspace => { editor.backspace(); }
                KeyCode::Delete => { editor.delete_char(); }
                KeyCode::Enter => { editor.newline(); }
                KeyCode::Tab => { editor.insert_tab(); }
                KeyCode::Char(c) if !ctrl => { editor.insert_char(c); }
                KeyCode::F(1) => { editor.show_about = true; }
                _ => {}
            }

            editor.scroll_to_view(edit_h);
            let _ = execute!(terminal.backend_mut(), SetTitle(&editor.title()));
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    println!("\nNotebook closed.");
    Ok(())
}
