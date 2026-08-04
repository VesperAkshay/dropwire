import re

with open("src/ui.rs", "r", encoding="utf-8") as f:
    code = f.read()

theme_struct = """
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
"""

if "ThemeColors" not in code:
    code = code.replace("use crate::app::{ActiveView, App};", "use crate::app::{ActiveView, App};\n" + theme_struct)

# Add Sparkline import
if "Sparkline" not in code:
    code = code.replace("Paragraph}", "Paragraph, Sparkline}")

# Inject theme colors in draw
if "let theme = ThemeColors::new(&app.theme);" not in code:
    code = code.replace("pub fn draw(f: &mut Frame, app: &mut App) {", "pub fn draw(f: &mut Frame, app: &mut App) {\n    let theme = ThemeColors::new(&app.theme);")

# Replace colors in draw
code = code.replace("Color::Rgb(254, 247, 228)", "theme.highlight")
code = code.replace("Color::Rgb(255, 184, 0)", "theme.primary")
code = code.replace("Color::Rgb(100, 100, 100)", "theme.border")
code = code.replace("Color::Rgb(60, 60, 60)", "theme.border")
code = code.replace("Color::Rgb(170, 144, 179)", "theme.secondary")

# Handle footer text for history and batch
code = code.replace(
    "ActiveView::FileBrowser => \" [↑/↓] Navigate  |  [Enter] Open Dir  |  [S] Send Selected  |  [R] Receive  |  [C] Open Config  |  [Q] Quit \",",
    "ActiveView::FileBrowser => \" [↑/↓] Navigate  |  [Space] Select  |  [Enter] Open Dir  |  [S] Send  |  [R] Receive  |  [H] History  |  [C] Config \","
)

if "ActiveView::History =>" not in code:
    code = code.replace(
        "ActiveView::LoadingScreen => \"\",",
        "ActiveView::LoadingScreen => \"\",\n        ActiveView::History => \" [↑/↓] Scroll  |  [Esc] Back \","
    )

if "ActiveView::History => draw_history" not in code:
    code = code.replace(
        "ActiveView::LoadingScreen => unreachable!(),",
        "ActiveView::LoadingScreen => unreachable!(),\n        ActiveView::History => draw_history(f, app, chunks[1], &theme),"
    )

# Fix draw_file_browser
if "let theme = ThemeColors::new(&app.theme);" not in code.split("fn draw_file_browser")[1].split("{")[0]:
    code = code.replace(
        "fn draw_file_browser(f: &mut Frame, app: &App, area: Rect) {",
        "fn draw_file_browser(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {"
    )
    code = code.replace(
        "draw_file_browser(f, app, chunks[1])",
        "draw_file_browser(f, app, chunks[1], &theme)"
    )

# Fix draw_receive_input
if "fn draw_receive_input(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors)" not in code:
    code = code.replace(
        "fn draw_receive_input(f: &mut Frame, app: &App, area: Rect) {",
        "fn draw_receive_input(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {"
    )
    code = code.replace(
        "draw_receive_input(f, app, chunks[1])",
        "draw_receive_input(f, app, chunks[1], &theme)"
    )

# Fix draw_transfer_dashboard
if "fn draw_transfer_dashboard(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors)" not in code:
    code = code.replace(
        "fn draw_transfer_dashboard(f: &mut Frame, app: &App, area: Rect) {",
        "fn draw_transfer_dashboard(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {"
    )
    code = code.replace(
        "draw_transfer_dashboard(f, app, chunks[1])",
        "draw_transfer_dashboard(f, app, chunks[1], &theme)"
    )

# Fix draw_config_editor
if "fn draw_config_editor(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors)" not in code:
    code = code.replace(
        "fn draw_config_editor(f: &mut Frame, app: &App, area: Rect) {",
        "fn draw_config_editor(f: &mut Frame, app: &App, area: Rect, theme: &ThemeColors) {"
    )
    code = code.replace(
        "draw_config_editor(f, app, chunks[1])",
        "draw_config_editor(f, app, chunks[1], &theme)"
    )

# Replace all hardcoded colors inside the functions
code = code.replace("Color::Rgb(254, 247, 228)", "theme.highlight")
code = code.replace("Color::Rgb(255, 184, 0)", "theme.primary")
code = code.replace("Color::Rgb(100, 100, 100)", "theme.border")
code = code.replace("Color::Rgb(60, 60, 60)", "theme.border")
code = code.replace("Color::Rgb(170, 144, 179)", "theme.secondary")

# Multi-select UI in FileBrowser
if "let is_selected_for_batch = app.selected_files.contains(path);" not in code:
    code = code.replace(
        "let icon = if path.is_dir() { \"📁\" } else { \"📄\" };",
        "let is_selected_for_batch = app.selected_files.contains(path);\n            let check = if is_selected_for_batch { \"[X]\" } else { \"[ ]\" };\n            let icon = if path.is_dir() { \"📁\" } else { \"📄\" };"
    )
    code = code.replace(
        "let name = path.file_name().unwrap_or_default().to_string_lossy();",
        "let name = path.file_name().unwrap_or_default().to_string_lossy();\n            let prefix = if name == \"..\" { \"\".to_string() } else { format!(\"{} \", check) };"
    )
    code = code.replace(
        "format!(\"  {} {}\", icon, name)",
        "format!(\"  {}{} {}\", prefix, icon, name)"
    )

# Sparkline in Dashboard
if "Sparkline" not in code.split("fn draw_transfer_dashboard")[1]:
    sparkline_injection = """
        let dash_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(5), Constraint::Length(3)].as_ref())
            .split(inner);
            
        f.render_widget(info_p, dash_chunks[0]);
        
        // Render Sparkline
        let data: Vec<u64> = st.speed_history.iter().map(|&v| v as u64).collect();
        let sparkline = Sparkline::default()
            .block(Block::default().title(" Speed History ").borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
            .data(&data)
            .style(Style::default().fg(theme.primary));
        f.render_widget(sparkline, dash_chunks[1]);
"""
    code = code.replace(
        "f.render_widget(info_p, inner);",
        sparkline_injection
    )

# Config options adding theme
if "Theme" not in code.split("fn draw_config_editor")[1]:
    code = code.replace(
        "let options = vec![",
        "let options = vec![\n        format!(\"Theme: {}\", app.config_state.theme),"
    )

# Adding History View Function
if "fn draw_history" not in code:
    code += """

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
"""

with open("src/ui.rs", "w", encoding="utf-8") as f:
    f.write(code)
print("Updated ui.rs successfully.")
