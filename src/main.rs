mod config;
mod twitter;
mod clipboard;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc;

use config::Config;
use twitter::TwitterClient;
use ui::{App, AppState};

enum PostCommand {
    Post { text: String, image_data: Option<Vec<u8>> },
}

enum PostResult {
    Success(String),
    Error(String),
}

#[tokio::main]
async fn main() -> Result<()> {

    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut image_data: Option<Vec<u8>> = None;

    let twitter_client = TwitterClient::new(config.twitter.clone());

    let (post_tx, mut post_rx) = mpsc::channel::<PostCommand>(10);
    let (result_tx, mut result_rx) = mpsc::channel::<PostResult>(10);

    let posting_task = tokio::spawn(async move {
        while let Some(cmd) = post_rx.recv().await {
            match cmd {
                PostCommand::Post { text, image_data } => {
                    let result = post_tweet(&twitter_client, text, image_data).await;
                    let _ = result_tx.send(result).await;
                }
            }
        }
    });

    let result = run_app(&mut terminal, &mut app, &mut image_data, post_tx, &mut result_rx).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    drop(posting_task);

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    image_data: &mut Option<Vec<u8>>,
    post_tx: mpsc::Sender<PostCommand>,
    result_rx: &mut mpsc::Receiver<PostResult>,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Ok(result) = result_rx.try_recv() {
            match result {
                PostResult::Success(tweet_id) => {
                    app.state = AppState::Success(tweet_id);
                }
                PostResult::Error(msg) => {
                    app.state = AppState::Error(msg);
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match app.state {
                    AppState::Composing => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Esc, _) => {
                                return Ok(());
                            }
                            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                                return Ok(());
                            }
                            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                                match clipboard::get_image_from_clipboard() {
                                    Ok(img_data) => {
                                        *image_data = Some(img_data);
                                        app.has_image = true;
                                    }
                                    Err(e) => {
                                        app.state = AppState::Error(format!("Clipboard error: {}", e));
                                    }
                                }
                            }
                            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                                app.state = AppState::FilePrompt;
                                app.file_path_input.clear();
                            }
                            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                                if !app.input.trim().is_empty() {
                                    app.state = AppState::Posting;
                                    let text = app.input.clone();
                                    let img_data = image_data.clone();
                                    let _ = post_tx.send(PostCommand::Post {
                                        text,
                                        image_data: img_data,
                                    }).await;
                                }
                            }
                            (KeyCode::Enter, _) => {
                                app.insert_char('\n');
                            }
                            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                                app.insert_char(c);
                            }
                            (KeyCode::Backspace, _) => {
                                app.delete_char();
                            }
                            (KeyCode::Left, _) => {
                                app.move_cursor_left();
                            }
                            (KeyCode::Right, _) => {
                                app.move_cursor_right();
                            }
                            _ => {}
                        }
                    }
                    AppState::FilePrompt => {
                        match key.code {
                            KeyCode::Esc => {
                                app.state = AppState::Composing;
                                app.file_path_input.clear();
                            }
                            KeyCode::Enter => {
                                let path = app.file_path_input.trim();
                                if !path.is_empty() {
                                    match clipboard::validate_image_file(path) {
                                        Ok(img_data) => {
                                            *image_data = Some(img_data);
                                            app.has_image = true;
                                            app.state = AppState::Composing;
                                            app.file_path_input.clear();
                                        }
                                        Err(e) => {
                                            app.state = AppState::Error(format!("Image error: {}", e));
                                        }
                                    }
                                } else {
                                    app.state = AppState::Composing;
                                }
                            }
                            KeyCode::Char(c) => {
                                app.file_path_input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.file_path_input.pop();
                            }
                            _ => {}
                        }
                    }
                    AppState::Posting => {
                    }
                    AppState::Success(_) | AppState::Error(_) => {
                        match key.code {
                            KeyCode::Esc => {
                                return Ok(());
                            }
                            _ => {
                                app.reset();
                                *image_data = None;
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn post_tweet(
    client: &TwitterClient,
    text: String,
    image_data: Option<Vec<u8>>,
) -> PostResult {
    let media_id = if let Some(img_data) = image_data {
        match client.upload_media(&img_data).await {
            Ok(id) => Some(id),
            Err(e) => {
                return PostResult::Error(format!("Failed to upload image: {}", e));
            }
        }
    } else {
        None
    };

    match client.post_tweet(text, media_id).await {
        Ok(tweet_data) => PostResult::Success(tweet_data.id),
        Err(e) => PostResult::Error(format!("Failed to post: {}", e)),
    }
}
