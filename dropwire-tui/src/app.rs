use std::path::PathBuf;
use std::fs;
use std::collections::{HashSet, VecDeque};

#[derive(Clone, PartialEq)]
pub enum ActiveView {
    LoadingScreen,
    FileBrowser,
    ReceiveInput,
    TransferDashboard,
    ConfigEditor,
    History,
}

#[derive(Clone, PartialEq)]
pub enum Theme {
    Cyberpunk,
    Matrix,
    Nord,
    Monochrome,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "matrix" => Theme::Matrix,
            "nord" => Theme::Nord,
            "monochrome" => Theme::Monochrome,
            _ => Theme::Cyberpunk,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Theme::Cyberpunk => "cyberpunk".to_string(),
            Theme::Matrix => "matrix".to_string(),
            Theme::Nord => "nord".to_string(),
            Theme::Monochrome => "monochrome".to_string(),
        }
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TransferHistoryEntry {
    pub date: String,
    pub filename: String,
    pub size: u64,
    pub speed_bps: f64,
    pub mode: String,
    pub is_send: bool,
}

impl TransferHistoryEntry {
    pub fn load_all() -> Vec<Self> {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dropwire");
        path.push("history.json");
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(history) = serde_json::from_str(&content) {
                return history;
            }
        }
        Vec::new()
    }
    pub fn save_all(history: &[Self]) {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("dropwire");
        let _ = fs::create_dir_all(&path);
        path.push("history.json");
        if let Ok(content) = serde_json::to_string_pretty(history) {
            let _ = fs::write(path, content);
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum BrowserPane {
    Sidebar,
    FileList,
}

#[derive(Clone)]
pub struct SidebarItem {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Clone)]
pub struct TransferState {
    pub is_sending: bool,
    pub code_phrase: String,
    pub status: String,
    pub current_bytes: u64,
    pub total_bytes: u64,
    pub start_time: std::time::Instant,
    pub last_time: std::time::Instant,
    pub last_bytes: u64,
    pub current_speed_bps: f64,
    pub speed_history: VecDeque<f64>,
}

pub struct ConfigState {
    pub relay: String,
    pub no_lan: bool,
    pub download_dir: String,
    pub default_mode: String,
    pub parallel_streams: String,
    pub chunk_size_kb: String,
    pub theme: String,
    pub selected_index: usize,
    pub is_editing: bool,
}

impl Default for ConfigState {
    fn default() -> Self {
        Self {
            relay: String::new(),
            no_lan: false,
            download_dir: String::new(),
            default_mode: "browser".to_string(),
            parallel_streams: "4".to_string(),
            chunk_size_kb: "1024".to_string(),
            theme: "cyberpunk".to_string(),
            selected_index: 0,
            is_editing: false,
        }
    }
}

pub struct App {
    pub view: ActiveView,
    pub should_quit: bool,
    pub current_dir: PathBuf,
    pub files: Vec<PathBuf>,
    pub selected_file_index: usize,
    pub selected_files: HashSet<PathBuf>,
    pub receive_code: String,
    pub transfer_state: Option<TransferState>,
    pub config_state: ConfigState,
    pub theme: Theme,
    pub history: Vec<TransferHistoryEntry>,
    pub history_scroll: usize,
    pub boot_time: std::time::Instant,
    pub next_view: Option<ActiveView>,
    pub current_transfer_task: Option<tokio::task::JoinHandle<()>>,
    pub browser_pane: BrowserPane,
    pub sidebar_items: Vec<SidebarItem>,
    pub selected_sidebar_index: usize,
}

impl App {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let config = dropwire::cli::config::DropwireConfig::load();
        let initial_view = if config.get_default_mode() == "receive" {
            ActiveView::ReceiveInput
        } else {
            ActiveView::FileBrowser
        };
        
        let theme = Theme::from_str(&config.get_theme());
        let history = TransferHistoryEntry::load_all();
        
        let mut config_state = ConfigState::default();
        config_state.relay = config.relay.clone().unwrap_or_default();
        config_state.no_lan = config.no_lan.clone().unwrap_or(false);
        config_state.download_dir = config.download_dir.clone().unwrap_or_default();
        config_state.default_mode = config.default_mode.clone().unwrap_or_default();
        config_state.parallel_streams = config.parallel_streams.clone().unwrap_or(4).to_string();
        config_state.chunk_size_kb = config.chunk_size_kb.clone().unwrap_or(1024).to_string();
        config_state.theme = config.get_theme();

        let mut sidebar_items = Vec::new();
        if let Some(d) = dirs::home_dir() { sidebar_items.push(SidebarItem { name: "Home".to_string(), path: d }); }
        if let Some(d) = dirs::desktop_dir() { sidebar_items.push(SidebarItem { name: "Desktop".to_string(), path: d }); }
        if let Some(d) = dirs::document_dir() { sidebar_items.push(SidebarItem { name: "Documents".to_string(), path: d }); }
        if let Some(d) = dirs::download_dir() { sidebar_items.push(SidebarItem { name: "Downloads".to_string(), path: d }); }
        if let Some(d) = dirs::picture_dir() { sidebar_items.push(SidebarItem { name: "Pictures".to_string(), path: d }); }
        if let Some(d) = dirs::audio_dir() { sidebar_items.push(SidebarItem { name: "Music".to_string(), path: d }); }
        if let Some(d) = dirs::video_dir() { sidebar_items.push(SidebarItem { name: "Videos".to_string(), path: d }); }
        
        #[cfg(windows)]
        {
            for letter in b'A'..=b'Z' {
                let path_str = format!("{}:\\", letter as char);
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    sidebar_items.push(SidebarItem { name: format!("Drive {}", letter as char), path });
                }
            }
        }
        #[cfg(unix)]
        {
            sidebar_items.push(SidebarItem { name: "Root".to_string(), path: PathBuf::from("/") });
        }

        let mut app = Self {
            view: ActiveView::LoadingScreen,
            next_view: Some(initial_view),
            boot_time: std::time::Instant::now(),
            should_quit: false,
            current_dir,
            files: Vec::new(),
            selected_file_index: 0,
            selected_files: HashSet::new(),
            receive_code: String::new(),
            transfer_state: None,
            config_state,
            theme,
            history,
            history_scroll: 0,
            current_transfer_task: None,
            browser_pane: BrowserPane::FileList,
            sidebar_items,
            selected_sidebar_index: 0,
        };
        app.refresh_dir();
        app
    }

    pub fn refresh_dir(&mut self) {
        self.files.clear();
        
        // Add parent directory option if not root
        if self.current_dir.parent().is_some() {
            self.files.push(PathBuf::from(".."));
        }

        if let Ok(entries) = fs::read_dir(&self.current_dir) {
            let mut paths: Vec<PathBuf> = entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .collect();
            
            // Sort directories first, then files
            paths.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                if a_is_dir == b_is_dir {
                    a.file_name().cmp(&b.file_name())
                } else if a_is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
            
            self.files.extend(paths);
        }
        self.selected_file_index = 0;
    }

    pub fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file_index = (self.selected_file_index + 1) % self.files.len();
        }
    }

    pub fn previous_file(&mut self) {
        if !self.files.is_empty() {
            if self.selected_file_index > 0 {
                self.selected_file_index -= 1;
            } else {
                self.selected_file_index = self.files.len() - 1;
            }
        }
    }

    pub fn browser_up(&mut self) {
        if self.browser_pane == BrowserPane::Sidebar {
            if !self.sidebar_items.is_empty() {
                if self.selected_sidebar_index > 0 {
                    self.selected_sidebar_index -= 1;
                } else {
                    self.selected_sidebar_index = self.sidebar_items.len() - 1;
                }
            }
        } else {
            self.previous_file();
        }
    }

    pub fn browser_down(&mut self) {
        if self.browser_pane == BrowserPane::Sidebar {
            if !self.sidebar_items.is_empty() {
                self.selected_sidebar_index = (self.selected_sidebar_index + 1) % self.sidebar_items.len();
            }
        } else {
            self.next_file();
        }
    }

    pub fn browser_enter(&mut self) {
        if self.browser_pane == BrowserPane::Sidebar {
            if !self.sidebar_items.is_empty() {
                let item = &self.sidebar_items[self.selected_sidebar_index];
                self.current_dir = item.path.clone();
                self.refresh_dir();
                self.browser_pane = BrowserPane::FileList;
            }
        } else {
            self.enter_selected();
        }
    }

    pub fn enter_selected(&mut self) {
        if self.files.is_empty() {
            return;
        }

        let selected = &self.files[self.selected_file_index];
        
        if selected == std::path::Path::new("..") {
            if let Some(parent) = self.current_dir.parent() {
                self.current_dir = parent.to_path_buf();
                self.refresh_dir();
            }
        } else if selected.is_dir() {
            // Because selected is an absolute path from fs::read_dir
            if let Ok(canonical) = selected.canonicalize() {
                self.current_dir = canonical;
            } else {
                self.current_dir = selected.clone();
            }
            self.refresh_dir();
        } else {
            // It's a file, we could default to sending it here as well, 
            // but we'll add an explicit send method.
        }
    }

    pub fn get_selected_path(&self) -> Option<PathBuf> {
        if self.files.is_empty() {
            return None;
        }
        let selected = &self.files[self.selected_file_index];
        if selected == std::path::Path::new("..") {
            None // Can't send the parent directory shortcut
        } else {
            Some(selected.clone())
        }
    }

    #[cfg(windows)]
    pub fn get_drives(&self) -> Vec<PathBuf> {
        let mut drives = Vec::new();
        for letter in b'A'..=b'Z' {
            let path_str = format!("{}:\\", letter as char);
            let path = PathBuf::from(&path_str);
            if path.exists() {
                drives.push(path);
            }
        }
        drives
    }

    #[cfg(unix)]
    pub fn get_drives(&self) -> Vec<PathBuf> {
        vec![PathBuf::from("/")]
    }

    pub fn cycle_drive(&mut self) {
        let drives = self.get_drives();
        if drives.is_empty() { return; }

        let current_root = self.current_dir.ancestors().last().unwrap_or(&self.current_dir).to_path_buf();
        
        let mut next_index = 0;
        for (i, drive) in drives.iter().enumerate() {
            if drive.to_string_lossy().to_uppercase().starts_with(&current_root.to_string_lossy().to_uppercase()) {
                next_index = (i + 1) % drives.len();
                break;
            }
        }
        
        self.current_dir = drives[next_index].clone();
        self.refresh_dir();
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
