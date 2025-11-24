use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Modifier},
    widgets::{Block, Borders, Paragraph, Wrap, List, ListItem, ListState},
    Frame,
};
use tui_textarea::TextArea;

use crate::drafts::Draft;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Composing,
    DraftBrowser,
    FilePrompt,
    Posting,
    Success(String), // Tweet URL
    Error(String),
}

pub struct App<'a> {
    pub state: AppState,
    pub textarea: TextArea<'a>,
    pub has_image: bool,
    pub file_path_input: String,
    pub drafts: Vec<Draft>,
    pub draft_list_state: ListState,
    pub current_draft_id: Option<String>,
}

impl<'a> App<'a> {
    pub fn new() -> Self {
        let mut textarea = TextArea::default();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Compose your post")
                .border_style(Style::default().fg(Color::Cyan)),
        );
        textarea.set_cursor_line_style(Style::default());
        textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        
        Self {
            state: AppState::Composing,
            textarea,
            has_image: false,
            file_path_input: String::new(),
            drafts: Vec::new(),
            draft_list_state: ListState::default(),
            current_draft_id: None,
        }
    }

    pub fn char_count(&self) -> usize {
        self.textarea.lines().join("\n").chars().count()
    }

    pub fn get_text(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn set_text(&mut self, text: String) {
        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        self.textarea = TextArea::new(lines);
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Compose your post")
                .border_style(Style::default().fg(Color::Cyan)),
        );
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

    pub fn reset(&mut self) {
        self.textarea = TextArea::default();
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title("Compose your post")
                .border_style(Style::default().fg(Color::Cyan)),
        );
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
        self.has_image = false;
        self.file_path_input.clear();
        self.state = AppState::Composing;
        self.current_draft_id = None;
    }

    pub fn load_drafts(&mut self) {
        if let Ok(drafts) = crate::drafts::load_drafts() {
            self.drafts = drafts;
            if !self.drafts.is_empty() {
                self.draft_list_state.select(Some(0));
            }
        }
    }

    pub fn next_draft(&mut self) {
        if self.drafts.is_empty() {
            return;
        }
        let i = match self.draft_list_state.selected() {
            Some(i) => {
                if i >= self.drafts.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.draft_list_state.select(Some(i));
    }

    pub fn previous_draft(&mut self) {
        if self.drafts.is_empty() {
            return;
        }
        let i = match self.draft_list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.drafts.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.draft_list_state.select(Some(i));
    }

    pub fn select_current_draft(&mut self) {
        if let Some(i) = self.draft_list_state.selected() {
            if let Some(draft) = self.drafts.get(i).cloned() {
                self.set_text(draft.content.clone());
                self.current_draft_id = Some(draft.id.clone());
                self.state = AppState::Composing;
            }
        }
    }

    pub fn delete_selected_draft(&mut self) {
        if let Some(i) = self.draft_list_state.selected() {
            if let Some(draft) = self.drafts.get(i) {
                let _ = crate::drafts::delete_draft(&draft.id);
                self.drafts.remove(i);
                
                // Update selection
                if self.drafts.is_empty() {
                    self.draft_list_state.select(None);
                } else if i >= self.drafts.len() {
                    self.draft_list_state.select(Some(self.drafts.len() - 1));
                }
            }
        }
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    if app.state == AppState::DraftBrowser {
        draw_draft_browser(f, app);
        return;
    }

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

fn draw_text_input(f: &mut Frame, app: &mut App, area: Rect) {
    if app.state == AppState::FilePrompt {
        let input = Paragraph::new(app.file_path_input.as_str())
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Enter image file path")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(input, area);
    } else {
        let title = match &app.state {
            AppState::Posting => "Posting...",
            _ => "Compose your post",
        };
        
        let mut textarea = app.textarea.clone();
        textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(&textarea, area);
    }
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
            let draft_indicator = if app.current_draft_id.is_some() {
                " | 📝 Draft loaded"
            } else {
                ""
            };
            
            format!("Characters: {}{}{}", char_count, image_indicator, draft_indicator)
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
        AppState::DraftBrowser => {
            format!("Drafts: {} saved", app.drafts.len())
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
            "Ctrl+U: upload image | Ctrl+S: save draft | Ctrl+D: drafts | Ctrl+P: post | Esc: exit"
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
        AppState::DraftBrowser => {
            "↑/↓: navigate | Enter: load draft | Delete: remove draft | Esc: back"
        }
    };

    let help = Paragraph::new(instructions)
        .style(Style::default().fg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Gray)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(help, area);
}

fn draw_draft_browser(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Draft list
    let items: Vec<ListItem> = app
        .drafts
        .iter()
        .map(|draft| {
            ListItem::new(draft.preview())
                .style(Style::default().fg(Color::White))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Saved Drafts")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    f.render_stateful_widget(list, chunks[0], &mut app.draft_list_state);
    
    draw_status(f, app, chunks[1]);
    draw_instructions(f, app, chunks[2]);
}
