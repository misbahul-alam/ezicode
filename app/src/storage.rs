use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::settings::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CursorPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenTabState {
    pub path: PathBuf,
    #[serde(default)]
    pub preview: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language_override: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<CursorPosition>,
}

fn default_sidebar_width() -> f32 {
    300.0
}

fn default_terminal_height() -> f32 {
    320.0
}

fn default_terminal_right_width() -> f32 {
    420.0
}

fn default_true() -> bool {
    true
}

fn default_activity() -> String {
    "Explorer".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutState {
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: f32,
    #[serde(default = "default_terminal_height")]
    pub terminal_height: f32,
    #[serde(default = "default_terminal_right_width")]
    pub terminal_right_width: f32,
    #[serde(default = "default_true")]
    pub show_sidebar: bool,
    #[serde(default)]
    pub show_terminal: bool,
    #[serde(default)]
    pub show_terminal_right: bool,
    #[serde(default)]
    pub terminal_maximized: bool,
    #[serde(default = "default_activity")]
    pub activity: String,
    /// VS Code's `workbench.tree.enableStickyScroll`, per workspace.
    #[serde(default = "default_true")]
    pub explorer_sticky_scroll: bool,
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_width: default_sidebar_width(),
            terminal_height: default_terminal_height(),
            terminal_right_width: default_terminal_right_width(),
            show_sidebar: default_true(),
            show_terminal: false,
            show_terminal_right: false,
            terminal_maximized: false,
            activity: default_activity(),
            explorer_sticky_scroll: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct WorkspaceState {
    pub root: PathBuf,
    #[serde(default)]
    pub tabs: Vec<OpenTabState>,
    #[serde(default)]
    pub active_tab: usize,
    #[serde(default)]
    pub layout: LayoutState,
    #[serde(default)]
    pub expanded_folders: Vec<PathBuf>,
    /// Focused explorer row, restored on the next session like VS Code does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explorer_selected: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GlobalState {
    #[serde(default)]
    pub recent_folders: Vec<PathBuf>,
    #[serde(default)]
    pub recent_files: Vec<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_workspace_root: Option<PathBuf>,
}

/// Returns the path to the `globalStorage` directory under ezicode's config directory.
pub fn global_storage_dir() -> PathBuf {
    config_dir().join("globalStorage")
}

/// Returns the path to the `workspaceStorage` directory under ezicode's config directory.
pub fn workspace_storage_base_dir() -> PathBuf {
    config_dir().join("workspaceStorage")
}

/// Returns the full path to `globalStorage/storage.json`.
pub fn global_storage_file() -> PathBuf {
    global_storage_dir().join("storage.json")
}

/// Computes a deterministic, filesystem-safe ID for a workspace root path.
/// Format: `<folder_name>-<hash>` (e.g. `my-project-7a8f3b2c1d0e4f5a`).
pub fn workspace_id(root: &Path) -> String {
    let canonical = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let folder_name = canonical
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("workspace");

    let clean_name: String = folder_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();

    format!("{clean_name}-{hash:016x}")
}

/// Returns the path to the workspace storage directory for a specific workspace root.
pub fn workspace_storage_dir(root: &Path) -> PathBuf {
    workspace_storage_base_dir().join(workspace_id(root))
}

/// Returns the path to the `state.json` file for a specific workspace root.
pub fn workspace_state_file(root: &Path) -> PathBuf {
    workspace_storage_dir(root).join("state.json")
}

fn write_json_safe<T: Serialize>(path: &Path, value: &T) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json.as_bytes())?;
    Ok(())
}

impl GlobalState {
    /// Loads global state from `storage.json`, returning default state if not found or invalid.
    pub fn load() -> Self {
        let path = global_storage_file();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<GlobalState>(&content) {
                return state;
            }
        }
        GlobalState::default()
    }

    /// Saves global state to `storage.json`.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = global_storage_file();
        write_json_safe(&path, self)
    }

    /// Adds a folder to the recent folders list (MRU order), deduplicating and limiting size.
    pub fn add_recent_folder(&mut self, folder: PathBuf) {
        let canonical = std::fs::canonicalize(&folder).unwrap_or(folder);
        self.recent_folders
            .retain(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()) != canonical);
        self.recent_folders.insert(0, canonical.clone());
        if self.recent_folders.len() > 30 {
            self.recent_folders.truncate(30);
        }
        self.last_workspace_root = Some(canonical);
        let _ = self.save();
    }

    /// Removes a folder from recent folders list.
    #[allow(dead_code)]
    pub fn remove_recent_folder(&mut self, folder: &Path) {
        let canonical = std::fs::canonicalize(folder).unwrap_or_else(|_| folder.to_path_buf());
        self.recent_folders
            .retain(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()) != canonical);
        if self
            .last_workspace_root
            .as_ref()
            .map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()))
            == Some(canonical)
        {
            self.last_workspace_root = self.recent_folders.first().cloned();
        }
        let _ = self.save();
    }

    /// Adds a file to the recent files list (MRU order), deduplicating and limiting size.
    pub fn add_recent_file(&mut self, file: PathBuf) {
        let canonical = std::fs::canonicalize(&file).unwrap_or(file);
        self.recent_files
            .retain(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.clone()) != canonical);
        self.recent_files.insert(0, canonical);
        if self.recent_files.len() > 50 {
            self.recent_files.truncate(50);
        }
        let _ = self.save();
    }
}

impl WorkspaceState {
    /// Loads workspace state for the given root folder.
    pub fn load(root: &Path) -> Option<Self> {
        let path = workspace_state_file(root);
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(state) = serde_json::from_str::<WorkspaceState>(&content) {
                return Some(state);
            }
        }
        None
    }

    /// Saves workspace state to its workspaceStorage folder.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = workspace_state_file(&self.root);
        write_json_safe(&path, self)
    }

    /// Deletes stored state for a workspace root.
    #[allow(dead_code)]
    pub fn delete(root: &Path) -> Result<(), std::io::Error> {
        let dir = workspace_storage_dir(root);
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_id_deterministic() {
        let path1 = PathBuf::from("/path/to/project");
        let path2 = PathBuf::from("/path/to/project");
        let path3 = PathBuf::from("/path/to/other");

        assert_eq!(workspace_id(&path1), workspace_id(&path2));
        assert_ne!(workspace_id(&path1), workspace_id(&path3));
        assert!(workspace_id(&path1).starts_with("project-"));
    }

    #[test]
    fn test_workspace_state_serde_roundtrip() {
        let state = WorkspaceState {
            root: PathBuf::from("/home/user/project"),
            tabs: vec![
                OpenTabState {
                    path: PathBuf::from("/home/user/project/src/main.rs"),
                    preview: false,
                    language_override: Some("rust".to_string()),
                    cursor: Some(CursorPosition {
                        line: 42,
                        character: 10,
                    }),
                },
                OpenTabState {
                    path: PathBuf::from("/home/user/project/README.md"),
                    preview: true,
                    language_override: None,
                    cursor: None,
                },
            ],
            active_tab: 0,
            layout: LayoutState {
                sidebar_width: 280.0,
                terminal_height: 250.0,
                terminal_right_width: 420.0,
                show_sidebar: true,
                show_terminal: true,
                show_terminal_right: true,
                terminal_maximized: false,
                activity: "Search".to_string(),
                explorer_sticky_scroll: true,
            },
            expanded_folders: vec![PathBuf::from("/home/user/project/src")],
            explorer_selected: Some(PathBuf::from("/home/user/project/src/main.rs")),
        };

        let json = serde_json::to_string_pretty(&state).unwrap();
        let deserialized: WorkspaceState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_global_state_serde_roundtrip() {
        let mut state = GlobalState::default();
        let folder1 = PathBuf::from("/tmp/folder1");
        let folder2 = PathBuf::from("/tmp/folder2");
        let file1 = PathBuf::from("/tmp/file1.rs");

        state.recent_folders.push(folder1.clone());
        state.recent_folders.push(folder2.clone());
        state.recent_files.push(file1.clone());
        state.last_workspace_root = Some(folder1);

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: GlobalState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_partial_json_defaults() {
        let partial_json = r#"{
            "root": "/tmp/test"
        }"#;

        let state: WorkspaceState = serde_json::from_str(partial_json).unwrap();
        assert_eq!(state.root, PathBuf::from("/tmp/test"));
        assert!(state.tabs.is_empty());
        assert_eq!(state.active_tab, 0);
        assert_eq!(state.layout.sidebar_width, 300.0);
        assert_eq!(state.layout.terminal_height, 320.0);
        assert_eq!(state.layout.terminal_right_width, 420.0);
        assert!(state.layout.show_sidebar);
        assert!(!state.layout.show_terminal);
        assert!(!state.layout.show_terminal_right);
        assert_eq!(state.layout.activity, "Explorer");
        assert!(state.expanded_folders.is_empty());
        assert!(state.layout.explorer_sticky_scroll);
        assert!(state.explorer_selected.is_none());
    }

    /// Workspace files written before the right terminal dock existed have no
    /// `terminal_right_width` / `show_terminal_right` keys. They must keep
    /// loading, with the dock's serde defaults filling the gap — a stale
    /// layout file can never brick a workspace.
    #[test]
    fn test_layout_without_right_dock_fields_still_loads() {
        let old_json = r#"{
            "root": "/tmp/legacy",
            "layout": {
                "sidebar_width": 280.0,
                "terminal_height": 250.0,
                "show_sidebar": true,
                "show_terminal": true,
                "terminal_maximized": false,
                "activity": "Explorer"
            }
        }"#;

        let state: WorkspaceState = serde_json::from_str(old_json).unwrap();
        assert_eq!(state.layout.sidebar_width, 280.0);
        assert_eq!(state.layout.terminal_height, 250.0);
        assert!(state.layout.show_terminal);
        // Right-dock defaults, not a parse error.
        assert_eq!(state.layout.terminal_right_width, 420.0);
        assert!(!state.layout.show_terminal_right);
        // Sticky scroll defaults on, exactly like VS Code.
        assert!(state.layout.explorer_sticky_scroll);
    }
}
