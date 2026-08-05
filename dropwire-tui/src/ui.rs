use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::app::{ActiveView, App};

pub struct ThemeColors {
    pub primary: Color,
    pub secondary: Color,
    pub highlight: Color,
    pub text: Color,
    pub border: Color,
}

impl ThemeColors {
    pub fn new(theme: &crate::app::Theme) -> Self {
        match theme {
            crate::app::Theme::Matrix => Self {
                primary: Color::Rgb(0, 255, 0),
                secondary: Color::Rgb(0, 150, 0),
                highlight: Color::Rgb(200, 255, 200),
                text: Color::Rgb(0, 200, 0),
                border: Color::Rgb(0, 50, 0),
            },
            crate::app::Theme::Nord => Self {
                primary: Color::Rgb(136, 192, 208),
                secondary: Color::Rgb(129, 161, 193),
                highlight: Color::Rgb(236, 239, 244),
                text: Color::Rgb(216, 222, 233),
                border: Color::Rgb(76, 86, 106),
            },
            crate::app::Theme::Monochrome => Self {
                primary: Color::Rgb(255, 255, 255),
                secondary: Color::Rgb(200, 200, 200),
                highlight: Color::Rgb(255, 255, 255),
                text: Color::Rgb(220, 220, 220),
                border: Color::Rgb(100, 100, 100),
            },
            crate::app::Theme::Cyberpunk => Self {
                primary: Color::Rgb(255, 184, 0),
                secondary: Color::Rgb(170, 144, 179),
                highlight: Color::Rgb(254, 247, 228),
                text: Color::Rgb(200, 200, 200),
                border: Color::Rgb(60, 60, 60),
            },
        }
    }
}


pub fn draw(f: &mut Frame, app: &mut App) {
    let theme = ThemeColors::new(&app.theme);
    if matches!(app.view, ActiveView::LoadingScreen) {
        draw_loading_screen(f, app, &theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(10),     // Header (ASCII art is 6 lines + subtitle + borders)
                Constraint::Min(10),        // Main content
                Constraint::Length(3),      // Footer
            ]
            .as_ref(),
        )
        .split(f.size());

    // --- Header ---
    let mut header_lines = vec![];
    let dropwire_art = [
        "██████╗ ██████╗  ██████╗ ██████╗ ██╗    ██╗██╗██████╗ ███████╗",
        "██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██║    ██║██║██╔══██╗██╔════╝",
        "██║  ██║██████╔╝██║   ██║██████╔╝██║ █╗ ██║██║██████╔╝█████╗  ",
        "██║  ██║██╔══██╗██║   ██║██╔═══╝ ██║███╗██║██║██╔══██╗██╔══╝  ",
        "██████╔╝██║  ██║╚██████╔╝██║     ╚███╔███╔╝██║██║  ██║███████╗",
        "╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝      ╚══╝╚══╝ ╚═╝╚═╝  ╚═╝╚══════╝",
    ];
    let x_art = [
        "██╗  ██╗",
        "╚██╗██╔╝",
        " ╚███╔╝ ",
        " ██╔██╗ ",
        "██╔╝ ██╗",
        "╚═╝  ╚═╝",
    ];

    for i in 0..6 {
        header_lines.push(Line::from(vec![
            Span::styled(dropwire_art[i], Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)),
            Span::styled(x_art[i], Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]));
    }
    
    header_lines.push(Line::from(vec![
        Span::styled("E2E ENCRYPTION  •  P2P TRANSPORT  •  ZERO-KNOWLEDGE RELAYS", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
    ]));

    let header = Paragraph::new(header_lines)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)));
    f.render_widget(header, chunks[0]);

    // --- Main Content ---
    match app.view {
        ActiveView::FileBrowser => draw_file_browser(f, app, chunks[1], &theme),
        ActiveView::ReceiveInput => draw_receive_input(f, app, chunks[1], &theme),
        ActiveView::TransferDashboard => draw_transfer_dashboard(f, app, chunks[1], &theme),
        ActiveView::ConfigEditor => draw_config_editor(f, app, chunks[1], &theme),
        ActiveView::LoadingScreen => unreachable!(),
        ActiveView::History => draw_history(f, app, chunks[1], &theme),
    }

    // --- Footer ---
    let footer_text = match app.view {
        ActiveView::FileBrowser => " [↑/↓] Nav  |  [Tab] Drive  |  [Space] Select  |  [Enter] Open  |  [S] Send  |  [R] Recv  |  [H] Hist  |  [C] Cfg ",
        ActiveView::ReceiveInput => " [Enter] Start Transfer  |  [Esc] Cancel  |  [Q] Quit ",
        ActiveView::TransferDashboard => " [Esc] Back to Explorer  |  [Q] Quit ",
        ActiveView::ConfigEditor => " [↑/↓] Select  |  [Enter] Edit/Toggle  |  [Esc] Save & Back ",
        ActiveView::LoadingScreen => "",
        ActiveView::History => " [↑/↓] Scroll  |  [Esc] Back ",
    };
    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
        .style(Style::default().fg(theme.secondary));
    f.render_widget(footer, chunks[2]);
}

fn draw_loading_screen(f: &mut Frame, app: &App, theme: &ThemeColors) {
    let area = f.size();
    
    // Create an artificial boot sequence text based on time
    let elapsed = app.boot_time.elapsed().as_secs_f32();
    let status_text = if elapsed < 0.4 {
        "INITIALIZING CRYPTOGRAPHIC ENGINE..."
    } else if elapsed < 0.8 {
        "GENERATING EPHEMERAL SESSION KEYS..."
    } else if elapsed < 1.2 {
        "BINDING TO NETWORK INTERFACES..."
    } else {
        "READY. SECURE P2P ENVIRONMENT ESTABLISHED."
    };

    let title_art = [
        "██████╗ ██████╗  ██████╗ ██████╗ ██╗    ██╗██╗██████╗ ███████╗    ██╗  ██╗",
        "██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██║    ██║██║██╔══██╗██╔════╝    ╚██╗██╔╝",
        "██║  ██║██████╔╝██║   ██║██████╔╝██║ █╗ ██║██║██████╔╝█████╗       ╚███╔╝ ",
        "██║  ██║██╔══██╗██║   ██║██╔═══╝ ██║███╗██║██║██╔══██╗██╔══╝       ██╔██╗ ",
        "██████╔╝██║  ██║╚██████╔╝██║     ╚███╔███╔╝██║██║  ██║███████╗    ██╔╝ ██╗",
        "╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝      ╚══╝╚══╝ ╚═╝╚═╝  ╚═╝╚══════╝    ╚═╝  ╚═╝",
    ];

    let mut lines = vec![];
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    for line in title_art {
        lines.push(Line::from(vec![
            Span::styled(line, Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD)),
        ]));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(status_text, Style::default().fg(Color::Rgb(0, 82, 255)).add_modifier(Modifier::BOLD))));
    
    // Progress bar animation
    let width = 40;
    let progress = ((elapsed / 1.5) * width as f32).min(width as f32) as usize;
    let bar = format!("[{}{}]{}", "█".repeat(progress), "░".repeat(width - progress), if elapsed >= 1.5 { " OK" } else { "" });
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(bar, Style::default().fg(Color::Rgb(0, 255, 255)).add_modifier(Modifier::BOLD))));

    let block = Paragraph::new(lines)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)));
    
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Length(15),
            Constraint::Percentage(30),
        ].as_ref())
        .split(area);
        
    f.render_widget(block, vertical_chunks[1]);
}

fn draw_file_browser(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    let mut list_items = Vec::new();
    
    for (i, path) in app.files.iter().enumerate() {
        let is_selected = i == app.selected_file_index;
        
        // Formatting the item
        let file_name = if path.to_str() == Some("..") {
            "../ (Parent Directory)".to_string()
        } else {
            path.file_name().unwrap_or_default().to_string_lossy().into_owned()
        };

        let (icon, color) = if path.is_dir() || path.to_str() == Some("..") {
            ("📁", theme.primary) // DropWire Gold
        } else {
            ("📄", Color::Rgb(200, 200, 200)) // Light Gray
        };

        let style = if is_selected {
            Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 82, 255)).add_modifier(Modifier::BOLD) // Electric Blue
        } else {
            Style::default().fg(color)
        };

        let content = format!(" {} {} ", icon, file_name);
        list_items.push(ListItem::new(content).style(style));
    }

    let title = format!(" Explorer: {} ", app.current_dir.display());
    let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(theme.border)));
    
    // We don't use ListState fully automatically here because we track index in App
    // But we need a ListState for the widget to render selection/scrolling correctly
    let mut state = ListState::default();
    state.select(Some(app.selected_file_index));
    
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_receive_input(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Length(3), Constraint::Percentage(40)].as_ref())
        .split(area);
        
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(50), Constraint::Percentage(25)].as_ref())
        .split(vertical_chunks[1]);
        
    let input_area = horizontal_chunks[1];
    
    let display_text = if app.receive_code.is_empty() {
        Span::styled(" e.g. 7-guitar-revenge", Style::default().fg(theme.border))
    } else {
        Span::styled(format!(" {}█", app.receive_code), Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD))
    };

    let input_widget = Paragraph::new(display_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary)) // Gold
                .title(" Enter Code Phrase to Receive ")
        );

    f.render_widget(input_widget, input_area);
    
    let help = Paragraph::new("Press [Enter] to connect  |  [Esc] to cancel")
        .alignment(ratatui::layout::Alignment::Center)
        .style(Style::default().fg(theme.secondary));
    
    f.render_widget(help, vertical_chunks[2]);
}

fn draw_transfer_dashboard(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    if let Some(state) = &app.transfer_state {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Length(6), // Progress bar (needs 6 lines for text + borders)
                Constraint::Percentage(40), // Spacing
            ].as_ref())
            .margin(2)
            .split(area);

        // Render code phrase big
        let mode = if state.is_sending { "SENDING" } else { "RECEIVING" };
        let title = Paragraph::new(vec![
            Line::from(vec![Span::styled(format!(" {} MODE ", mode), Style::default().fg(theme.secondary))]),
            Line::from(""),
            Line::from(vec![Span::styled(state.code_phrase.clone(), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
            Line::from(""),
            Line::from(vec![Span::styled(state.status.clone(), Style::default().fg(Color::Rgb(255, 255, 255)))]),
        ])
        .alignment(ratatui::layout::Alignment::Center);
        
        f.render_widget(title, chunks[0]);

        // Render progress bar
        let speed_mbps = state.current_speed_bps / 1_048_576.0;
        let speed_kbps = state.current_speed_bps / 1024.0;
        let speed_str = if speed_mbps < 1.0 {
            format!("{:.1} KB/s", speed_kbps)
        } else {
            format!("{:.1} MB/s", speed_mbps)
        };
        
        let ratio = if state.total_bytes > 0 {
            (state.current_bytes as f64 / state.total_bytes as f64).clamp(0.0, 1.0)
        } else {
            0.0
        };
        
        let percent = (ratio * 100.0) as u32;
        let width = 45;
        let filled = (ratio * width as f64).round() as usize;
        let empty = width - filled;
        let bar_str = format!("{}{}", "█".repeat(filled), "░".repeat(empty));
        
        let progress_text = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!(" {} ", bar_str), Style::default().fg(Color::Rgb(0, 255, 128))),
                Span::styled(format!(" {:>3}% ", percent), Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD)),
                Span::styled(format!(" |  {}", speed_str), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(format!("  {} / {} encrypted chunks verified", state.current_bytes, state.total_bytes), Style::default().fg(theme.border))]),
        ];

        let gauge = Paragraph::new(progress_text)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)));
            
        f.render_widget(gauge, chunks[1]);
        
    } else {
        let p = Paragraph::new("Initializing DropWire Engine...")
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border)));
        f.render_widget(p, area);
    }
}

fn draw_config_editor(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    let mut items = vec![];
    
    let relay_style = if app.config_state.selected_index == 0 { Style::default().bg(Color::Rgb(0, 82, 255)).fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default() };
    let relay_text = if app.config_state.selected_index == 0 && app.config_state.is_editing {
        format!(" [Relay URL]: {}_ ", app.config_state.relay)
    } else {
        format!(" [Relay URL]: {} ", app.config_state.relay)
    };
    items.push(ListItem::new(relay_text).style(relay_style));

    let no_lan_style = if app.config_state.selected_index == 1 { Style::default().bg(Color::Rgb(0, 82, 255)).fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default() };
    items.push(ListItem::new(format!(" [Disable LAN]: {} ", app.config_state.no_lan)).style(no_lan_style));

    let dl_style = if app.config_state.selected_index == 2 { Style::default().bg(Color::Rgb(0, 82, 255)).fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default() };
    let dl_text = if app.config_state.selected_index == 2 && app.config_state.is_editing {
        format!(" [Download Dir]: {}_ ", app.config_state.download_dir)
    } else {
        let dir_display = if app.config_state.download_dir.is_empty() { "<Default / Downloads/Dropwire>" } else { &app.config_state.download_dir };
        format!(" [Download Dir]: {} ", dir_display)
    };
    items.push(ListItem::new(dl_text).style(dl_style));

    let mode_style = if app.config_state.selected_index == 3 { Style::default().bg(Color::Rgb(0, 82, 255)).fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default() };
    let mode_text = format!(" [Default Mode (browser/receive)]: {} ", app.config_state.default_mode);
    items.push(ListItem::new(mode_text).style(mode_style));

    let streams_style = if app.config_state.selected_index == 4 { Style::default().bg(Color::Rgb(0, 82, 255)).fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default() };
    let streams_text = if app.config_state.selected_index == 4 && app.config_state.is_editing {
        format!(" [Parallel Streams (4-16)]: {}_ ", app.config_state.parallel_streams)
    } else {
        format!(" [Parallel Streams (4-16)]: {} ", app.config_state.parallel_streams)
    };
    items.push(ListItem::new(streams_text).style(streams_style));

    let chunk_style = if app.config_state.selected_index == 5 { Style::default().bg(Color::Rgb(0, 82, 255)).fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default() };
    let chunk_text = if app.config_state.selected_index == 5 && app.config_state.is_editing {
        format!(" [Chunk Size KB]: {}_ ", app.config_state.chunk_size_kb)
    } else {
        format!(" [Chunk Size KB]: {} ", app.config_state.chunk_size_kb)
    };
    items.push(ListItem::new(chunk_text).style(chunk_style));

    let theme_style = if app.config_state.selected_index == 6 { Style::default().bg(Color::Rgb(0, 82, 255)).fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default() };
    let theme_text = format!(" [Theme]: {} (Press Enter to cycle) ", app.config_state.theme);
    items.push(ListItem::new(theme_text).style(theme_style));

    let list = List::new(items)
        .block(Block::default().title(" Configuration ").borders(Borders::ALL).border_style(Style::default().fg(theme.primary)));
        
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(0)].as_ref())
        .margin(2)
        .split(area);
        
    // Needs ListState to track scrolling in case terminal gets tiny
    let mut state = ratatui::widgets::ListState::default();
    state.select(Some(app.config_state.selected_index));
    f.render_stateful_widget(list, chunks[0], &mut state);
}


fn draw_history(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {
    let block = Block::default()
        .title(" Transfer History ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));
        
    let inner = block.inner(area);
    f.render_widget(block, area);
    
    if app.history.is_empty() {
        let empty = Paragraph::new("No history found.")
            .style(Style::default().fg(theme.secondary))
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(empty, inner);
        return;
    }
    
    let mut items = vec![];
    for (i, entry) in app.history.iter().enumerate() {
        let style = if i == app.history_scroll {
            Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };
        
        let direction = if entry.is_send { "Sent" } else { "Received" };
        let text = format!("{} | {} | {} | {:.2} MB | {:.2} MB/s", 
            entry.date, direction, entry.filename, entry.size as f64 / 1_048_576.0, entry.speed_bps / 1_048_576.0);
            
        items.push(ListItem::new(text).style(style));
    }
    
    let list = List::new(items);
    let mut state = ListState::default();
    state.select(Some(app.history_scroll));
    f.render_stateful_widget(list, inner, &mut state);
}
