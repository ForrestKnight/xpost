use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Composing,
    FilePrompt,
    Posting,
    Success(String), // Tweet URL
    Error(String),
}

pub struct App {
    pub state: AppState,
    pub input: String,
    pub cursor_position: usize,
    pub has_image: bool,
    pub file_path_input: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Composing,
            input: String::new(),
            cursor_position: 0,
            has_image: false,
            file_path_input: String::new(),
        }
    }

    pub fn char_count(&self) -> usize {
        self.input.chars().count()
    }

    pub fn insert_char(&mut self, c: char) {
        self.input.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.input.remove(self.cursor_position - 1);
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.has_image = false;
        self.file_path_input.clear();
        self.state = AppState::Composing;
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_text_input(f, app, chunks[0]);
    draw_status(f, app, chunks[1]);
    draw_instructions(f, app, chunks[2]);
}

fn draw_text_input(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.state == AppState::FilePrompt {
        &app.file_path_input
    } else {
        &app.input
    };

    let title = match &app.state {
        AppState::FilePrompt => "Enter image file path",
        AppState::Posting => "Posting...",
        _ => "Compose your post",
    };

    let input = Paragraph::new(text.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(input, area);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let status_text = match &app.state {
        AppState::Composing => {
            let char_count = app.char_count();
            let image_indicator = if app.has_image {
                " | 📎 Image attached"
            } else {
                ""
            };
            
            format!("Characters: {}{}", char_count, image_indicator)
        }
        AppState::FilePrompt => {
            "Enter the path to your image file".to_string()
        }
        AppState::Posting => {
            "Posting to X...".to_string()
        }
        AppState::Success(url) => {
            format!("✓ Posted successfully! https://x.com/user/status/{}", url)
        }
        AppState::Error(msg) => {
            format!("✗ Error: {}", msg)
        }
    };

    let status_color = match &app.state {
        AppState::Success(_) => Color::Green,
        AppState::Error(_) => Color::Red,
        AppState::Posting => Color::Yellow,
        _ => Color::White,
    };

    let status = Paragraph::new(status_text)
        .style(Style::default().fg(status_color))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status")
                .border_style(Style::default().fg(Color::Gray)),
        );

    f.render_widget(status, area);
}

fn draw_instructions(f: &mut Frame, app: &App, area: Rect) {
    let instructions = match &app.state {
        AppState::Composing => {
            "Ctrl+V: paste image | Ctrl+U: upload image | Ctrl+P: post | Esc: exit"
        }
        AppState::FilePrompt => {
            "Enter: confirm | Esc: cancel"
        }
        AppState::Posting => {
            "Please wait..."
        }
        AppState::Success(_) | AppState::Error(_) => {
            "Press any key to post again, or Esc to exit"
        }
    };

    let help = Paragraph::new(instructions)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        );

    f.render_widget(help, area);
}
