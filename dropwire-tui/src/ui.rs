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

    pub fn animated_primary(&self, app: &App) -> Color {
        let t = app.boot_time.elapsed().as_secs_f32() * 3.0; // speed
        
        match app.theme {
            crate::app::Theme::Cyberpunk => {
                // Morph between Gold (255, 184, 0) and Red/Magenta (255, 50, 100)
                let r = 255;
                let g = (184.0 - ((t.sin() * 0.5 + 0.5) * 134.0)) as u8;
                let b = ((t.sin() * 0.5 + 0.5) * 100.0) as u8;
                Color::Rgb(r, g, b)
            },
            crate::app::Theme::Matrix => {
                // Pulse green brightness
                let g = (255.0 - ((t.sin().abs()) * 100.0)) as u8;
                Color::Rgb(0, g, 0)
            },
            crate::app::Theme::Nord => {
                // Shift between blue and cyan
                let r = (136.0 + ((t.cos() * 0.5) * 40.0)) as u8;
                let g = (192.0 + ((t.sin() * 0.5) * 40.0)) as u8;
                Color::Rgb(r, g, 255)
            },
            crate::app::Theme::Monochrome => {
                // Pulse white/gray
                let v = (255.0 - ((t.sin().abs()) * 80.0)) as u8;
                Color::Rgb(v, v, v)
            }
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
        ActiveView::FileBrowser => " [↑/↓] Nav  |  [Tab/◄/►] Panes  |  [Space] Select  |  [Enter] Open  |  [S] Send  |  [R] Recv  |  [H] Hist  |  [C] Cfg  |  [Q] Quit ",
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
    use ratatui::widgets::canvas::{Canvas, Points};
    let area = f.size();
    
    let elapsed = app.boot_time.elapsed().as_secs_f32();
    let total_time = 3.5;
    let assemble_time = 2.5;
    let progress = (elapsed / assemble_time).clamp(0.0, 1.0);
    
    // Smooth cubic ease out
    let ease = 1.0 - (1.0 - progress).powi(3);

    let title_art = [
        "██████╗ ██████╗  ██████╗ ██████╗ ██╗    ██╗██╗██████╗ ███████╗    ██╗  ██╗",
        "██╔══██╗██╔══██╗██╔═══██╗██╔══██╗██║    ██║██║██╔══██╗██╔════╝    ╚██╗██╔╝",
        "██║  ██║██████╔╝██║   ██║██████╔╝██║ █╗ ██║██║██████╔╝█████╗       ╚███╔╝ ",
        "██║  ██║██╔══██╗██║   ██║██╔═══╝ ██║███╗██║██║██╔══██╗██╔══╝       ██╔██╗ ",
        "██████╔╝██║  ██║╚██████╔╝██║     ╚███╔███╔╝██║██║  ██║███████╗    ██╔╝ ██╗",
        "╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝      ╚══╝╚══╝ ╚═╝╚═╝  ╚═╝╚══════╝    ╚═╝  ╚═╝",
    ];

    if progress < 0.95 {
        let w = area.width as f64;
        let h = area.height as f64;
        let art_w = 74.0;
        let art_h = 6.0;
        let start_x = (w - art_w) / 2.0;
        let start_y = (h - art_h) / 2.0 + 3.0; // Slightly higher to account for subtitle

        let mut final_points = vec![];
        for (y, line) in title_art.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                if ch != ' ' {
                    let base_x = start_x + x as f64;
                    let base_y = start_y + (5.0 - y as f64);
                    
                    if ch == '█' || ch == '║' || ch == '╗' || ch == '╚' || ch == '╝' || ch == '╔' || ch == '═' {
                        final_points.push((base_x + 0.25, base_y + 0.25));
                        final_points.push((base_x + 0.75, base_y + 0.25));
                        final_points.push((base_x + 0.25, base_y + 0.75));
                        final_points.push((base_x + 0.75, base_y + 0.75));
                    } else {
                        final_points.push((base_x + 0.5, base_y + 0.5));
                    }
                }
            }
        }

        let mut current_points = Vec::with_capacity(final_points.len());
        let mut trail_points = Vec::new();
        
        for &(fx, fy) in &final_points {
            let hash = fx * 17.0 + fy * 31.0;
            let dist = 80.0 + (hash % 50.0);
            let sx = fx + hash.cos() * dist;
            let sy = fy + hash.sin() * dist;
            
            let cx = sx + (fx - sx) * ease as f64;
            let cy = sy + (fy - sy) * ease as f64;
            current_points.push((cx, cy));
            
            if progress > 0.05 && progress < 1.0 {
                let cx_trail = sx + (fx - sx) * (ease as f64 * 0.85);
                let cy_trail = sy + (fy - sy) * (ease as f64 * 0.85);
                trail_points.push((cx_trail, cy_trail));
            }
        }

        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::NONE))
            .x_bounds([0.0, w])
            .y_bounds([0.0, h])
            .paint(move |ctx| {
                if !trail_points.is_empty() {
                    ctx.draw(&Points {
                        coords: &trail_points,
                        color: theme.secondary,
                    });
                }
                ctx.draw(&Points {
                    coords: &current_points,
                    color: theme.animated_primary(app),
                });
                
                if progress > 0.7 {
                    let sub_y = (start_y - 3.0) - 2.0; 
                    ctx.print(
                        (w - 38.0) / 2.0, sub_y,
                        Span::styled("E2E ENCRYPTION  •  P2P TRANSPORT", Style::default().fg(theme.border).add_modifier(Modifier::BOLD))
                    );
                }
            });
            
        f.render_widget(canvas, area);
    } else {
        // Draw the perfectly crisp raw ASCII text once the particles have fully combined
        let mut header_lines = vec![];
        header_lines.push(Line::from(""));
        header_lines.push(Line::from(""));

        for line in title_art {
            header_lines.push(Line::from(vec![
                Span::styled(line, Style::default().fg(theme.animated_primary(app)).add_modifier(Modifier::BOLD)),
            ]));
        }
        
        header_lines.push(Line::from(""));
        header_lines.push(Line::from(vec![
            Span::styled("E2E ENCRYPTION  •  P2P TRANSPORT", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        ]));

        let block = Paragraph::new(header_lines)
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::NONE));
        
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
}


fn draw_file_browser(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(25), Constraint::Min(0)].as_ref())
        .split(area);

    let sidebar_area = chunks[0];
    let list_area = chunks[1];

    let mut sidebar_list_items = Vec::new();
    for (i, item) in app.sidebar_items.iter().enumerate() {
        let is_selected = i == app.selected_sidebar_index;
        let style = if is_selected && app.browser_pane == crate::app::BrowserPane::Sidebar {
            Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 82, 255)).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::Rgb(200, 200, 200)).bg(Color::Rgb(50, 50, 50))
        } else {
            Style::default().fg(Color::Rgb(200, 200, 200))
        };
        let content = format!(" {} ", item.name);
        sidebar_list_items.push(ListItem::new(content).style(style));
    }

    let sidebar_title = if app.browser_pane == crate::app::BrowserPane::Sidebar { " Places (Active) " } else { " Places " };
    let sidebar_border_style = if app.browser_pane == crate::app::BrowserPane::Sidebar { Style::default().fg(theme.animated_primary(app)) } else { Style::default().fg(theme.border) };
    let sidebar_list = List::new(sidebar_list_items)
        .block(Block::default().title(sidebar_title).borders(Borders::ALL).border_style(sidebar_border_style));
    let mut sidebar_state = ListState::default();
    sidebar_state.select(Some(app.selected_sidebar_index));
    f.render_stateful_widget(sidebar_list, sidebar_area, &mut sidebar_state);


    let mut list_items = Vec::new();
    
    for (i, path) in app.files.iter().enumerate() {
        let is_selected = i == app.selected_file_index;
        
        let file_name = if path.to_str() == Some("..") {
            "../ (Parent Directory)".to_string()
        } else {
            path.file_name().unwrap_or_default().to_string_lossy().into_owned()
        };

        let (icon, color) = if path.is_dir() || path.to_str() == Some("..") {
            ("📁", theme.primary)
        } else {
            ("📄", Color::Rgb(200, 200, 200))
        };

        let style = if is_selected && app.browser_pane == crate::app::BrowserPane::FileList {
            Style::default().fg(Color::Rgb(255, 255, 255)).bg(Color::Rgb(0, 82, 255)).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::Rgb(200, 200, 200)).bg(Color::Rgb(50, 50, 50))
        } else {
            Style::default().fg(color)
        };

        let content = format!(" {} {} ", icon, file_name);
        list_items.push(ListItem::new(content).style(style));
    }

    let title = if app.browser_pane == crate::app::BrowserPane::FileList { format!(" Explorer (Active): {} ", app.current_dir.display()) } else { format!(" Explorer: {} ", app.current_dir.display()) };
    let border_style = if app.browser_pane == crate::app::BrowserPane::FileList { Style::default().fg(theme.animated_primary(app)) } else { Style::default().fg(theme.border) };
    let list = List::new(list_items)
        .block(Block::default().title(title).borders(Borders::ALL).border_style(border_style));
    
    let mut state = ListState::default();
    state.select(Some(app.selected_file_index));
    
    f.render_stateful_widget(list, list_area, &mut state);
}

fn draw_receive_input(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    // Full centered layout: top spacer / header card / input / hint / bottom spacer
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Length(7),  // header card
            Constraint::Length(1),  // gap
            Constraint::Length(3),  // input
            Constraint::Length(2),  // hint
            Constraint::Percentage(15),
        ].as_ref())
        .split(area);

    // Narrow column in the center for all widgets
    let col = |row: Rect| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)].as_ref())
            .split(row)[1]
    };

    // ── Header card ──────────────────────────────────────────────────────────
    let header_area = col(root[1]);
    let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let spinner_idx = (app.boot_time.elapsed().as_millis() / 80) as usize % spinners.len();
    let spinner = spinners[spinner_idx];

    let header = Paragraph::new(vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {}  RECEIVE MODE", spinner), Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter the code phrase shared by the sender", Style::default().fg(Color::Rgb(150, 150, 150))),
        ]),
        Line::from(""),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .title(Span::styled(" DropWire ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)))
    );
    f.render_widget(header, header_area);

    // ── Input field ──────────────────────────────────────────────────────────
    let input_area = col(root[3]);
    let elapsed = app.boot_time.elapsed().as_millis();
    let show_cursor = (elapsed % 1000) < 500;

    let display_text = if app.receive_code.is_empty() {
        let placeholder = if show_cursor { " e.g. 7-guitar-revenge █" } else { " e.g. 7-guitar-revenge  " };
        Span::styled(placeholder, Style::default().fg(theme.border))
    } else {
        let text = if show_cursor {
            format!(" {}█", app.receive_code)
        } else {
            format!(" {} ", app.receive_code)
        };
        Span::styled(text, Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD))
    };

    let input_widget = Paragraph::new(Line::from(vec![display_text]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.animated_primary(app)))
                .title(Span::styled(" Code Phrase ", Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD)))
        )
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(input_widget, input_area);

    // ── Hint ─────────────────────────────────────────────────────────────────
    let hint_area = col(root[4]);
    let hint = Paragraph::new(Line::from(vec![
        Span::styled("[Enter]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" Connect   ", Style::default().fg(Color::Rgb(120, 120, 120))),
        Span::styled("[Esc]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" Cancel", Style::default().fg(Color::Rgb(120, 120, 120))),
    ]))
    .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(hint, hint_area);
}

fn draw_transfer_dashboard(f: &mut Frame, app: &mut App, area: Rect, theme: &ThemeColors) {
    if let Some(state) = &app.transfer_state {
        // Root vertical layout
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Length(9),  // info card
                Constraint::Length(1),  // gap
                Constraint::Length(3),  // progress gauge
                Constraint::Length(1),  // gap
                Constraint::Length(3),  // stats bar
                Constraint::Percentage(10),
            ].as_ref())
            .margin(2)
            .split(area);

        // Centre column helper
        let col = |row: Rect| {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(15), Constraint::Percentage(70), Constraint::Percentage(15)].as_ref())
                .split(row)[1]
        };

        // ── Spinner / Mode / Code ───────────────────────────────────────────
        let spinners = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let spinner_idx = (app.boot_time.elapsed().as_millis() / 80) as usize % spinners.len();
        let spinner = spinners[spinner_idx];
        let mode = if state.is_sending { "SENDING" } else { "RECEIVING" };
        let mode_color = if state.is_sending { Color::Rgb(255, 184, 0) } else { Color::Rgb(80, 200, 255) };

        let info_card = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(format!("  {}  {} MODE", spinner, mode), Style::default().fg(mode_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Code: ", Style::default().fg(Color::Rgb(120, 120, 120))),
                Span::styled(state.code_phrase.clone(), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Status: ", Style::default().fg(Color::Rgb(120, 120, 120))),
                Span::styled(state.status.clone(), Style::default().fg(Color::Rgb(255, 255, 255))),
            ]),
            Line::from(""),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title(Span::styled(" Transfer ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)))
        );
        f.render_widget(info_card, col(root[1]));

        // ── Progress gauge ──────────────────────────────────────────────────
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

        let gauge = ratatui::widgets::Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.animated_primary(app)))
                    .title(Span::styled(" Progress ", Style::default().fg(theme.highlight)))
            )
            .gauge_style(Style::default().fg(theme.primary).bg(Color::Rgb(30, 30, 30)))
            .ratio(ratio)
            .label(format!("{:.1}%   {}", ratio * 100.0, speed_str));
        f.render_widget(gauge, col(root[3]));

        // ── Stats bar ───────────────────────────────────────────────────────
        let stats_area = col(root[5]);
        let bytes_done = state.current_bytes;
        let bytes_total = state.total_bytes;
        let stats = Paragraph::new(Line::from(vec![
            Span::styled(" Verified: ", Style::default().fg(Color::Rgb(120, 120, 120))),
            Span::styled(format!("{} B", bytes_done), Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled("  /  ", Style::default().fg(Color::Rgb(80, 80, 80))),
            Span::styled(format!("{} B", bytes_total), Style::default().fg(Color::Rgb(160, 160, 160))),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
        )
        .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(stats, stats_area);

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
