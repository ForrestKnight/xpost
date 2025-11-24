use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::twitter::Tweet;

#[derive(Debug, Clone)]
pub enum StatsState {
    TweetList,
    StatsDetail,
    Loading(String),
    Error(String),
}

pub struct StatsApp {
    pub state: StatsState,
    pub tweets: Vec<Tweet>,
    pub selected_index: usize,
    pub list_state: ListState,
    pub replies: Vec<Tweet>,
    pub scroll_offset: usize,
}

impl StatsApp {
    pub fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            state: StatsState::Loading("Fetching tweets...".to_string()),
            tweets: Vec::new(),
            selected_index: 0,
            list_state,
            replies: Vec::new(),
            scroll_offset: 0,
        }
    }

    pub fn set_tweets(&mut self, tweets: Vec<Tweet>) {
        self.tweets = tweets;
        if !self.tweets.is_empty() {
            self.state = StatsState::TweetList;
            self.selected_index = 0;
            self.list_state.select(Some(0));
        } else {
            self.state = StatsState::Error("No tweets found".to_string());
        }
    }

    pub fn next(&mut self) {
        if self.tweets.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.tweets.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    pub fn previous(&mut self) {
        if self.tweets.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tweets.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.selected_index = i;
    }

    pub fn get_selected_tweet(&self) -> Option<&Tweet> {
        self.tweets.get(self.selected_index)
    }

    pub fn set_replies(&mut self, replies: Vec<Tweet>) {
        self.replies = replies;
        self.scroll_offset = 0;
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_offset < self.replies.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }
}

pub fn draw(f: &mut Frame, app: &mut StatsApp) {
    match &app.state {
        StatsState::TweetList => draw_tweet_list(f, app),
        StatsState::StatsDetail => draw_stats_detail(f, app),
        StatsState::Loading(msg) => draw_centered_message(f, msg, Color::Yellow),
        StatsState::Error(msg) => draw_centered_message(f, msg, Color::Red),
    }
}

fn draw_tweet_list(f: &mut Frame, app: &mut StatsApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("Your Recent Posts")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Tweet list
    let items: Vec<ListItem> = app
        .tweets
        .iter()
        .map(|tweet| {
            let text_preview = if tweet.text.len() > 80 {
                format!("{}...", &tweet.text[..80])
            } else {
                tweet.text.clone()
            };
            
            let date = tweet
                .created_at
                .as_ref()
                .map(|d| &d[..10])
                .unwrap_or("Unknown date");
            
            let content = format!("{} | {}", date, text_preview);
            ListItem::new(content)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Posts"))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[1], &mut app.list_state);

    // Footer
    let footer = Paragraph::new("↑/↓: Navigate | Enter: View Stats | Esc: Exit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[2]);
}

fn draw_stats_detail(f: &mut Frame, app: &StatsApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(12),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new("Post Statistics")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Tweet text
    if let Some(tweet) = app.get_selected_tweet() {
        let tweet_text = Paragraph::new(tweet.text.as_str())
            .wrap(Wrap { trim: false })
            .block(Block::default().borders(Borders::ALL).title("Post Content"))
            .style(Style::default().fg(Color::White));
        f.render_widget(tweet_text, chunks[1]);

        // Stats
        if let Some(metrics) = &tweet.public_metrics {
            let stats_text = vec![
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Likes: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{}", metrics.like_count)),
                ]),
                Line::from(vec![
                    Span::styled("  Retweets: ", Style::default().fg(Color::Green)),
                    Span::raw(format!("{}", metrics.retweet_count)),
                ]),
                Line::from(vec![
                    Span::styled("  Replies: ", Style::default().fg(Color::Cyan)),
                    Span::raw(format!("{}", metrics.reply_count)),
                ]),
                Line::from(vec![
                    Span::styled("  Quotes: ", Style::default().fg(Color::Magenta)),
                    Span::raw(format!("{}", metrics.quote_count)),
                ]),
                Line::from(vec![
                    Span::styled("  Impressions: ", Style::default().fg(Color::Blue)),
                    Span::raw(format!("{}", metrics.impression_count)),
                ]),
            ];

            let stats = Paragraph::new(stats_text)
                .block(Block::default().borders(Borders::ALL).title("Metrics"))
                .alignment(Alignment::Left);
            f.render_widget(stats, chunks[2]);
        } else {
            let no_metrics = Paragraph::new("No metrics available")
                .block(Block::default().borders(Borders::ALL).title("Metrics"))
                .style(Style::default().fg(Color::Red));
            f.render_widget(no_metrics, chunks[2]);
        }
    }

    // Footer
    let footer = Paragraph::new("Esc: Back to List | Q: Exit")
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[3]);
}

fn draw_centered_message(f: &mut Frame, message: &str, color: Color) {
    let area = centered_rect(60, 20, f.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(color));
    let paragraph = Paragraph::new(message)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
