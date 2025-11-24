mod config;
mod twitter;
mod clipboard;
mod ui;
mod stats_ui;
mod drafts;

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
    let args: Vec<String> = std::env::args().collect();
    
    let config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    // Check if stats mode is requested
    if args.len() > 1 && args[1] == "stats" {
        return run_stats_mode(config).await;
    }

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

async fn run_app<'a>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &'a mut ui::App<'a>,
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
            let event = event::read()?;
            match event {
                Event::Key(key) => {
                match app.state {
                    AppState::Composing => {
                        match (key.code, key.modifiers) {
                            (KeyCode::Esc, _) => {
                                return Ok(());
                            }
                            (KeyCode::Char('c'), m) if m == KeyModifiers::CONTROL => {
                                return Ok(());
                            }
                            (KeyCode::Char('c'), m) if m == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                                // Copy text - handled by TextArea
                                app.textarea.input(key);
                            }
                            (KeyCode::Char('v'), m) if m == (KeyModifiers::CONTROL | KeyModifiers::SHIFT) => {
                                // Paste text - handled by TextArea
                                app.textarea.input(key);
                            }
                            (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                                app.state = AppState::FilePrompt;
                                app.file_path_input.clear();
                            }
                            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                                let text = app.get_text();
                                if !text.trim().is_empty() {
                                    let draft = if let Some(draft_id) = &app.current_draft_id {
                                        // Update existing draft
                                        if let Some(existing) = app.drafts.iter_mut().find(|d| &d.id == draft_id) {
                                            existing.update_content(text.clone());
                                            existing.clone()
                                        } else {
                                            drafts::Draft::new(text)
                                        }
                                    } else {
                                        // Create new draft
                                        drafts::Draft::new(text)
                                    };
                                    
                                    if let Err(e) = drafts::save_draft(&draft) {
                                        app.state = AppState::Error(format!("Failed to save draft: {}", e));
                                    } else {
                                        app.current_draft_id = Some(draft.id.clone());
                                    }
                                }
                            }
                            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                                app.load_drafts();
                                app.state = AppState::DraftBrowser;
                            }
                            (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                                let text = app.get_text();
                                if !text.trim().is_empty() {
                                    app.state = AppState::Posting;
                                    let img_data = image_data.clone();
                                    let _ = post_tx.send(PostCommand::Post {
                                        text,
                                        image_data: img_data,
                                    }).await;
                                }
                            }
                            _ => {
                                // Pass all other events to TextArea
                                app.textarea.input(key);
                            }
                        }
                    }
                    AppState::DraftBrowser => {
                        match key.code {
                            KeyCode::Esc => {
                                app.state = AppState::Composing;
                            }
                            KeyCode::Down => {
                                app.next_draft();
                            }
                            KeyCode::Up => {
                                app.previous_draft();
                            }
                            KeyCode::Enter => {
                                app.select_current_draft();
                            }
                            KeyCode::Delete => {
                                app.delete_selected_draft();
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
                Event::Mouse(mouse) => {
                    // Pass mouse events to TextArea for click-to-position and drag-to-select
                    if app.state == AppState::Composing {
                        app.textarea.input(crossterm::event::Event::Mouse(mouse));
                    }
                }
                _ => {}
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

async fn run_stats_mode(config: Config) -> Result<()> {
    let twitter_client = TwitterClient::new(config.twitter.clone());

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = stats_ui::StatsApp::new();

    // Fetch user info and tweets in background
    let client_clone = TwitterClient::new(config.twitter.clone());
    let (data_tx, mut data_rx) = mpsc::channel::<Result<Vec<twitter::Tweet>>>(1);
    
    tokio::spawn(async move {
        let result = async {
            let user = client_clone.get_current_user().await?;
            let tweets = client_clone.get_user_tweets(&user.id, 20).await?;
            Ok(tweets)
        }.await;
        let _ = data_tx.send(result).await;
    });

    let result = run_stats_app(&mut terminal, &mut app, &twitter_client, &mut data_rx).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_stats_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut stats_ui::StatsApp,
    twitter_client: &TwitterClient,
    data_rx: &mut mpsc::Receiver<Result<Vec<twitter::Tweet>>>,
) -> Result<()> {
    loop {
        terminal.draw(|f| stats_ui::draw(f, app))?;

        // Check for initial data load
        if let Ok(result) = data_rx.try_recv() {
            match result {
                Ok(tweets) => app.set_tweets(tweets),
                Err(e) => {
                    app.state = stats_ui::StatsState::Error(format!("Failed to load tweets: {}", e));
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match &app.state {
                    stats_ui::StatsState::TweetList => {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                return Ok(());
                            }
                            KeyCode::Down => {
                                app.next();
                            }
                            KeyCode::Up => {
                                app.previous();
                            }
                            KeyCode::Enter => {
                                app.state = stats_ui::StatsState::StatsDetail;
                            }
                            _ => {}
                        }
                    }
                    stats_ui::StatsState::StatsDetail => {
                        match key.code {
                            KeyCode::Esc => {
                                app.state = stats_ui::StatsState::TweetList;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
                    stats_ui::StatsState::Loading(_) => {
                        // Wait for loading to complete
                    }
                    stats_ui::StatsState::Error(_) => {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {
                                app.state = stats_ui::StatsState::TweetList;
                            }
                        }
                    }
                }
            }
        }
    }
}
