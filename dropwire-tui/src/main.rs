pub mod app;
pub mod ui;
pub mod engine;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io, time::Duration};
use tokio::sync::mpsc;

use app::{App, ActiveView, TransferState};
use engine::EngineEvent;

pub enum AppEvent {
    Input(CEvent),
    Engine(EngineEvent),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    
    // Input polling task
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(evt) = event::read() {
                    if tx_clone.send(AppEvent::Input(evt)).is_err() {
                        break;
                    }
                }
            }
            // Small yield to prevent blocking
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if let Ok(evt) = rx.try_recv() {
            match evt {
                AppEvent::Input(CEvent::Key(key)) => {
                    if key.kind == event::KeyEventKind::Press {
                        match app.view {
                            ActiveView::FileBrowser => {
                                match key.code {
                                    KeyCode::Char('q') => app.quit(),
                                    KeyCode::Char('r') | KeyCode::Char('R') => {
                                        app.view = ActiveView::ReceiveInput;
                                    }
                                    KeyCode::Up => app.browser_up(),
                                    KeyCode::Down => app.browser_down(),
                                    KeyCode::Left => app.browser_pane = crate::app::BrowserPane::Sidebar,
                                    KeyCode::Right => app.browser_pane = crate::app::BrowserPane::FileList,
                                    KeyCode::Tab => {
                                        app.browser_pane = if app.browser_pane == crate::app::BrowserPane::Sidebar {
                                            crate::app::BrowserPane::FileList
                                        } else {
                                            crate::app::BrowserPane::Sidebar
                                        };
                                    }
                                    KeyCode::Enter => app.browser_enter(),
                                    KeyCode::Char('s') | KeyCode::Char('S') => {
                                        let mut paths = app.selected_files.iter().cloned().collect::<Vec<_>>();
                                        if paths.is_empty() {
                                            if let Some(path) = app.get_selected_path() {
                                                paths.push(path);
                                            }
                                        }
                                        if !paths.is_empty() {
                                            app.view = ActiveView::TransferDashboard;
                                            app.current_transfer_task = Some(tokio::spawn(engine::start_send(paths, tx.clone())));
                                        }
                                    }
                                    KeyCode::Char(' ') => {
                                        if let Some(path) = app.get_selected_path() {
                                            if path.file_name().unwrap_or_default() != ".." {
                                                if app.selected_files.contains(&path) {
                                                    app.selected_files.remove(&path);
                                                } else {
                                                    app.selected_files.insert(path);
                                                }
                                                app.next_file();
                                            }
                                        }
                                    }
                                    KeyCode::Char('h') | KeyCode::Char('H') => {
                                        app.history = crate::app::TransferHistoryEntry::load_all();
                                        app.view = ActiveView::History;
                                    }
                                    KeyCode::Char('c') | KeyCode::Char('C') => {
                                        let config = dropwire::cli::config::DropwireConfig::load();
                                        app.config_state.relay = config.get_relay();
                                        app.config_state.no_lan = config.get_no_lan();
                                        app.config_state.download_dir = config.get_download_dir().map(|d| d.to_string_lossy().to_string()).unwrap_or_default();
                                        app.config_state.default_mode = config.get_default_mode();
                                        app.config_state.parallel_streams = config.get_parallel_streams().to_string();
                                        app.config_state.chunk_size_kb = config.get_chunk_size_kb().to_string();
                                        app.config_state.theme = config.get_theme();
                                        app.config_state.selected_index = 0;
                                        app.config_state.is_editing = false;
                                        app.view = ActiveView::ConfigEditor;
                                    }
                                    _ => {}
                                }
                            }
                            ActiveView::ConfigEditor => {
                                if app.config_state.is_editing {
                                    match key.code {
                                        KeyCode::Esc | KeyCode::Enter => {
                                            app.config_state.is_editing = false;
                                        }
                                        KeyCode::Backspace => {
                                            if app.config_state.selected_index == 0 {
                                                app.config_state.relay.pop();
                                            } else if app.config_state.selected_index == 2 {
                                                app.config_state.download_dir.pop();
                                            } else if app.config_state.selected_index == 4 {
                                                app.config_state.parallel_streams.pop();
                                            } else if app.config_state.selected_index == 5 {
                                                app.config_state.chunk_size_kb.pop();
                                            }
                                        }
                                        KeyCode::Char(c) => {
                                            if app.config_state.selected_index == 0 {
                                                app.config_state.relay.push(c);
                                            } else if app.config_state.selected_index == 2 {
                                                app.config_state.download_dir.push(c);
                                            } else if app.config_state.selected_index == 4 {
                                                app.config_state.parallel_streams.push(c);
                                            } else if app.config_state.selected_index == 5 {
                                                app.config_state.chunk_size_kb.push(c);
                                            }
                                        }
                                        _ => {}
                                    }
                                } else {
                                    match key.code {
                                        KeyCode::Esc => {
                                            let mut cfg = dropwire::cli::config::DropwireConfig::default();
                                            cfg.relay = Some(app.config_state.relay.clone());
                                            cfg.no_lan = Some(app.config_state.no_lan);
                                            cfg.download_dir = if app.config_state.download_dir.trim().is_empty() { None } else { Some(app.config_state.download_dir.clone()) };
                                            cfg.default_mode = Some(app.config_state.default_mode.clone());
                                            if let Ok(streams) = app.config_state.parallel_streams.parse::<u8>() {
                                                cfg.parallel_streams = Some(streams);
                                            }
                                            if let Ok(chunk_size) = app.config_state.chunk_size_kb.parse::<u32>() {
                                                cfg.chunk_size_kb = Some(chunk_size);
                                            }
                                            cfg.theme = Some(app.config_state.theme.clone());
                                            app.theme = crate::app::Theme::from_str(&app.config_state.theme);
                                            let _ = cfg.save();
                                            
                                            app.view = ActiveView::FileBrowser;
                                        }
                                        KeyCode::Up => {
                                            if app.config_state.selected_index > 0 { app.config_state.selected_index -= 1; }
                                        }
                                        KeyCode::Down => {
                                            if app.config_state.selected_index < 6 { app.config_state.selected_index += 1; }
                                        }
                                        KeyCode::Enter => {
                                            if app.config_state.selected_index == 1 {
                                                app.config_state.no_lan = !app.config_state.no_lan;
                                            } else if app.config_state.selected_index == 3 {
                                                if app.config_state.default_mode == "browser" {
                                                    app.config_state.default_mode = "receive".to_string();
                                                } else {
                                                    app.config_state.default_mode = "browser".to_string();
                                                }
                                            } else if app.config_state.selected_index == 6 {
                                                let themes = ["cyberpunk", "matrix", "nord", "monochrome"];
                                                let current_idx = themes.iter().position(|&t| t == app.config_state.theme.to_lowercase().as_str()).unwrap_or(0);
                                                app.config_state.theme = themes[(current_idx + 1) % themes.len()].to_string();
                                            } else {
                                                app.config_state.is_editing = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            ActiveView::ReceiveInput => {
                                match key.code {
                                    KeyCode::Esc => {
                                        app.view = ActiveView::FileBrowser;
                                    }
                                    KeyCode::Enter => {
                                        if !app.receive_code.trim().is_empty() {
                                            app.view = ActiveView::TransferDashboard;
                                            app.current_transfer_task = Some(tokio::spawn(engine::start_receive(
                                                app.receive_code.trim().to_string(), 
                                                app.current_dir.clone(), 
                                                tx.clone()
                                            )));
                                        }
                                    }
                                    KeyCode::Backspace => { app.receive_code.pop(); }
                                    KeyCode::Char(c) => { app.receive_code.push(c); }
                                    _ => {}
                                }
                            }
                            ActiveView::TransferDashboard => {
                                if key.code == KeyCode::Char('q') { app.quit(); }
                                if key.code == KeyCode::Esc { 
                                    if let Some(task) = app.current_transfer_task.take() {
                                        task.abort();
                                    }
                                    app.view = ActiveView::FileBrowser; 
                                    app.transfer_state = None; 
                                }
                            }
                            ActiveView::History => {
                                match key.code {
                                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        app.view = ActiveView::FileBrowser;
                                    }
                                    KeyCode::Up => {
                                        if app.history_scroll > 0 {
                                            app.history_scroll -= 1;
                                        }
                                    }
                                    KeyCode::Down => {
                                        if app.history_scroll + 1 < app.history.len() {
                                            app.history_scroll += 1;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            ActiveView::LoadingScreen => {} // Ignore keyboard during loading
                        }
                    }
                }
                AppEvent::Engine(engine_evt) => {
                    match engine_evt {
                        EngineEvent::InitSend(code) => {
                            let now = std::time::Instant::now();
                            app.transfer_state = Some(TransferState {
                                is_sending: true,
                                code_phrase: code,
                                status: "Initializing...".into(),
                                current_bytes: 0,
                                total_bytes: 0,
                                start_time: now,
                                last_time: now,
                                last_bytes: 0,
                                current_speed_bps: 0.0,
                                speed_history: std::collections::VecDeque::new(),
                            });
                        }
                        EngineEvent::InitReceive(code) => {
                            let now = std::time::Instant::now();
                            app.transfer_state = Some(TransferState {
                                is_sending: false,
                                code_phrase: code,
                                status: "Initializing...".into(),
                                current_bytes: 0,
                                total_bytes: 0,
                                start_time: now,
                                last_time: now,
                                last_bytes: 0,
                                current_speed_bps: 0.0,
                                speed_history: std::collections::VecDeque::new(),
                            });
                        }
                        EngineEvent::Status(s) => {
                            if let Some(st) = &mut app.transfer_state { st.status = s; }
                        }
                        EngineEvent::Progress { current, total } => {
                            if let Some(st) = &mut app.transfer_state {
                                st.current_bytes = current;
                                st.total_bytes = total;
                                
                                let elapsed = st.last_time.elapsed().as_secs_f64();
                                if elapsed >= 0.5 {
                                    let diff = current.saturating_sub(st.last_bytes) as f64;
                                    let config = dropwire::cli::config::DropwireConfig::load();
                                    let chunk_bytes = config.get_chunk_size_kb() as f64 * 1024.0;
                                    
                                    st.current_speed_bps = (diff * chunk_bytes) / elapsed;
                                    st.speed_history.push_back(st.current_speed_bps);
                                    if st.speed_history.len() > 60 {
                                        st.speed_history.pop_front();
                                    }
                                    st.last_bytes = current;
                                    st.last_time = std::time::Instant::now();
                                }
                            }
                        }
                        EngineEvent::Error(e) => {
                            if let Some(st) = &mut app.transfer_state { st.status = format!("ERROR: {}", e); }
                        }
                        EngineEvent::Done => {
                            if let Some(st) = &app.transfer_state {
                                let dt = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                                let avg_speed = if st.start_time.elapsed().as_secs_f64() > 0.0 {
                                    st.total_bytes as f64 / st.start_time.elapsed().as_secs_f64()
                                } else {
                                    0.0
                                };
                                let filename = if !app.selected_files.is_empty() {
                                    format!("Batch ({} items)", app.selected_files.len())
                                } else {
                                    app.get_selected_path().map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned()).unwrap_or_default()
                                };
                                let entry = crate::app::TransferHistoryEntry {
                                    date: dt,
                                    filename,
                                    size: st.total_bytes,
                                    speed_bps: avg_speed,
                                    mode: app.config_state.default_mode.clone(),
                                    is_send: st.is_sending,
                                };
                                app.history.push(entry);
                                crate::app::TransferHistoryEntry::save_all(&app.history);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        if matches!(app.view, ActiveView::LoadingScreen) {
            if app.boot_time.elapsed().as_secs_f32() >= 1.0 {
                if let Some(next) = app.next_view.take() {
                    app.view = next;
                }
            }
        }
        
        if app.should_quit {
            return Ok(());
        }
        
        // 60fps max render loop
        tokio::time::sleep(Duration::from_millis(16)).await;
    }
}
