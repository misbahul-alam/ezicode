mod colors;

pub use colors::Colors;

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::{json, Value};

use self::colors::{parse_hex, FALLBACKS, KEY_MAP};

const THEME_FILES: &[&str] = &["themes/github.json"];

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,

    pub appearance: String,
    pub colors: Colors,
    pub terminal_palette: gpui_terminal::ColorPalette,

    hl_json: String,
    /// Lazily-parsed `hl_json`. `highlight_theme()` is called on every theme
    /// switch, and re-deserializing the same JSON string each time is pure
    /// waste — parse once per `Theme` and clone the result (the struct is
    /// small: ~50 `Hsla` values).
    hl_cache: OnceLock<gpui_component::highlighter::HighlightTheme>,
}

fn hex_to_rgb(hex: u32) -> (u8, u8, u8) {
    (
        ((hex >> 24) & 0xff) as u8,
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
    )
}

fn build_terminal_palette(theme_obj: &Value) -> gpui_terminal::ColorPalette {
    let mut builder = gpui_terminal::ColorPalette::builder();
    let style = theme_obj.get("style").and_then(Value::as_object);

    let get_color = |key: &str| -> Option<(u8, u8, u8)> {
        style
            .and_then(|s| s.get(key))
            .and_then(Value::as_str)
            .and_then(parse_hex)
            .map(hex_to_rgb)
    };

    if let Some((r, g, b)) = get_color("terminal.background")
        .or_else(|| get_color("editor.background"))
        .or_else(|| get_color("background"))
    {
        builder = builder.background(r, g, b);
    }

    if let Some((r, g, b)) = get_color("terminal.foreground")
        .or_else(|| get_color("editor.foreground"))
        .or_else(|| get_color("text"))
    {
        builder = builder.foreground(r, g, b);
    }

    let cursor_color = theme_obj
        .get("players")
        .and_then(Value::as_array)
        .and_then(|p| p.first())
        .and_then(|p| p.get("cursor"))
        .and_then(Value::as_str)
        .and_then(parse_hex)
        .map(hex_to_rgb)
        .or_else(|| get_color("text.accent"))
        .or_else(|| get_color("terminal.bright_foreground"))
        .or_else(|| get_color("terminal.foreground"));

    if let Some((r, g, b)) = cursor_color {
        builder = builder.cursor(r, g, b);
    }

    let ansi_keys = [
        ("terminal.ansi.black", 0),
        ("terminal.ansi.red", 1),
        ("terminal.ansi.green", 2),
        ("terminal.ansi.yellow", 3),
        ("terminal.ansi.blue", 4),
        ("terminal.ansi.magenta", 5),
        ("terminal.ansi.cyan", 6),
        ("terminal.ansi.white", 7),
        ("terminal.ansi.bright_black", 8),
        ("terminal.ansi.bright_red", 9),
        ("terminal.ansi.bright_green", 10),
        ("terminal.ansi.bright_yellow", 11),
        ("terminal.ansi.bright_blue", 12),
        ("terminal.ansi.bright_magenta", 13),
        ("terminal.ansi.bright_cyan", 14),
        ("terminal.ansi.bright_white", 15),
    ];

    for (key, idx) in ansi_keys {
        if let Some((r, g, b)) = get_color(key) {
            builder = match idx {
                0 => builder.black(r, g, b),
                1 => builder.red(r, g, b),
                2 => builder.green(r, g, b),
                3 => builder.yellow(r, g, b),
                4 => builder.blue(r, g, b),
                5 => builder.magenta(r, g, b),
                6 => builder.cyan(r, g, b),
                7 => builder.white(r, g, b),
                8 => builder.bright_black(r, g, b),
                9 => builder.bright_red(r, g, b),
                10 => builder.bright_green(r, g, b),
                11 => builder.bright_yellow(r, g, b),
                12 => builder.bright_blue(r, g, b),
                13 => builder.bright_magenta(r, g, b),
                14 => builder.bright_cyan(r, g, b),
                15 => builder.bright_white(r, g, b),
                _ => builder,
            };
        }
    }

    builder.build()
}

fn parse_family(json: &str, out: &mut Vec<Theme>) {
    let Ok(v) = serde_json::from_str::<Value>(json) else {
        return;
    };
    let Some(themes) = v.get("themes").and_then(Value::as_array) else {
        return;
    };
    for t in themes {
        let Some(name) = t.get("name").and_then(Value::as_str) else {
            continue;
        };
        let appearance = t
            .get("appearance")
            .and_then(Value::as_str)
            .unwrap_or("dark")
            .to_string();

        let Some(style) = t.get("style").and_then(Value::as_object) else {
            continue;
        };
        let mut colors = Colors::all_missing();
        for (field, key) in KEY_MAP {
            if let Some(hex_str) = style.get(*key).and_then(Value::as_str) {
                if let Some(hex) = parse_hex(hex_str) {
                    if let Some(slot) = colors.get_mut(field) {
                        *slot = hex;
                    }
                }
            }
        }
        for (field, hex) in FALLBACKS {
            if let Some(slot) = colors.get_mut(field) {
                if *slot == Colors::MISSING {
                    *slot = *hex;
                }
            }
        }
        if colors.border == Colors::MISSING {
            colors.border = 0x30363dff;
        }
        if colors.border_variant == Colors::MISSING {
            colors.border_variant = if colors.border != Colors::MISSING {
                colors.border
            } else {
                0x21262dff
            };
        }

        if colors.tab_bar == Colors::MISSING {
            colors.tab_bar = if colors.toolbar != Colors::MISSING {
                colors.toolbar
            } else if colors.panel != Colors::MISSING {
                colors.panel
            } else if colors.surface != Colors::MISSING {
                colors.surface
            } else {
                colors.background
            };
        }
        if colors.tab_active_bg == Colors::MISSING {
            colors.tab_active_bg = if colors.editor_bg != Colors::MISSING {
                colors.editor_bg
            } else {
                colors.background
            };
        }
        if colors.tab_inactive_bg == Colors::MISSING {
            colors.tab_inactive_bg = colors.tab_bar;
        }
        if colors.tab_active_fg == Colors::MISSING {
            colors.tab_active_fg = if colors.editor_fg != Colors::MISSING {
                colors.editor_fg
            } else {
                colors.text
            };
        }
        if colors.tab_inactive_fg == Colors::MISSING {
            colors.tab_inactive_fg = if colors.text_muted != Colors::MISSING {
                colors.text_muted
            } else {
                colors.text
            };
        }
        if colors.terminal_bg == Colors::MISSING {
            colors.terminal_bg = if colors.background != Colors::MISSING {
                colors.background
            } else {
                colors.tab_bar
            };
        }
        // Final safety net: anything still undefined (typical for minimal
        // user themes) falls back to the base palette for this appearance —
        // Zed's `refine_theme_style` behavior. Runs last so the contextual
        // derivations above always take precedence.
        colors.refine_from_base(&appearance);
        out.push(Theme {
            name: name.to_string(),
            appearance,
            hl_json: build_highlight_theme(name, t),
            hl_cache: OnceLock::new(),
            terminal_palette: build_terminal_palette(t),
            colors,
        });
    }
}

fn build_highlight_theme(name: &str, theme_obj: &Value) -> String {
    let style = theme_obj.get("style").cloned().unwrap_or(json!({}));
    let hl = json!({
        "name": name,
        "appearance": theme_obj.get("appearance").cloned().unwrap_or(json!("dark")),
        "style": {
            "editor.background": style.get("editor.background"),
            "editor.foreground": style.get("editor.foreground"),
            "editor.active_line.background": style.get("editor.active_line.background"),
            "editor.line_number": style.get("editor.line_number"),
            "editor.active_line_number": style.get("editor.active_line_number"),
            "error": style.get("text").or(style.get("editor.foreground")),
            "error.background": style.get("error.background").or(style.get("elevated_surface.background")).or(style.get("panel.background")),
            "error.border": style.get("error.border").or(style.get("border.variant")).or(style.get("border")),
            "warning": style.get("text").or(style.get("editor.foreground")),
            "warning.background": style.get("warning.background").or(style.get("elevated_surface.background")).or(style.get("panel.background")),
            "warning.border": style.get("warning.border").or(style.get("border.variant")).or(style.get("border")),
            "info": style.get("text").or(style.get("editor.foreground")),
            "info.background": style.get("info.background").or(style.get("elevated_surface.background")).or(style.get("panel.background")),
            "info.border": style.get("info.border").or(style.get("border.variant")).or(style.get("border")),
            "hint": style.get("text").or(style.get("editor.foreground")),
            "hint.background": style.get("hint.background").or(style.get("elevated_surface.background")).or(style.get("panel.background")),
            "hint.border": style.get("hint.border").or(style.get("border.variant")),
            "syntax": style.get("syntax"),
        },
    });
    hl.to_string()
}

impl Theme {
    /// Tree-sitter token styles for this theme.
    ///
    /// Deserialized from the Zed JSON on first use, then cached in
    /// `hl_cache` — theme switches re-read this, and re-parsing the same
    /// JSON on every switch would be wasted work.
    pub fn highlight_theme(&self) -> gpui_component::highlighter::HighlightTheme {
        self.hl_cache
            .get_or_init(|| {
                serde_json::from_str(&self.hl_json).unwrap_or_else(|_| {
                    (*gpui_component::highlighter::HighlightTheme::default_dark()).clone()
                })
            })
            .clone()
    }
}

/// Parse one theme file, accepting both layouts:
///
/// - a Zed theme **family**: `{"name": …, "author": …, "themes": […]}` — the
///   format of every embedded asset and of Zed's own `~/.config/zed/themes`
///   files (a family may bundle a dark and a light variant); and
/// - a bare **single theme**: `{"name": …, "appearance": …, "style": …}` —
///   wrapped into a one-entry family so both layouts run through the exact
///   same token extraction, fallback, and terminal-palette pipeline.
///
/// Malformed files are skipped silently: a bad user file must never take the
/// editor down or break the embedded themes.
fn parse_theme_file(json: &str, out: &mut Vec<Theme>) {
    let Ok(value) = serde_json::from_str::<Value>(json) else {
        return;
    };
    if value.get("themes").is_some() {
        parse_family(json, out);
    } else if value.get("name").is_some() && value.get("style").is_some() {
        let family = json!({ "themes": [value] });
        parse_family(&family.to_string(), out);
    }
}

/// Directory users drop extra Zed-format theme JSON files into. Same contract
/// as Zed's `~/.config/zed/themes`: any `*.json` in here is merged into the
/// theme list (see [`load_user_themes_from`]) and shows up in the theme
/// picker, settings page, and command palette like a built-in.
pub fn user_themes_dir() -> PathBuf {
    crate::settings::config_dir().join("themes")
}

/// Load every `*.json` theme file in `dir` (sorted by file name, so the theme
/// order is deterministic between runs) into `out`.
///
/// A user theme that shares a name with an embedded theme **replaces** it —
/// the same override rule Zed applies to user themes — so users can restyle
/// the built-ins without recompiling. New names are appended after the
/// embedded families.
pub fn load_user_themes_from(dir: &Path, out: &mut Vec<Theme>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect();
    paths.sort();
    for path in paths {
        let Ok(json) = std::fs::read_to_string(&path) else {
            continue;
        };
        let before = out.len();
        parse_theme_file(&json, out);
        // Merge with override semantics: a theme that names an existing one
        // takes over its slot instead of appending a duplicate.
        let fresh: Vec<Theme> = out.drain(before..).collect();
        for theme in fresh {
            match out.iter().position(|existing| existing.name == theme.name) {
                Some(ix) => out[ix] = theme,
                None => out.push(theme),
            }
        }
    }
}

pub fn all() -> &'static [Theme] {
    static THEMES: OnceLock<Vec<Theme>> = OnceLock::new();
    THEMES.get_or_init(|| {
        let mut out = Vec::new();
        for file in THEME_FILES {
            if let Some(data) = crate::assets::AppAssets::get(file) {
                parse_family(std::str::from_utf8(&data.data).unwrap_or(""), &mut out);
            }
        }
        // User themes load after the embedded families so they can override
        // built-ins by name. The directory is created on first run (best
        // effort) so it is discoverable next to settings.json.
        let user_dir = user_themes_dir();
        let _ = std::fs::create_dir_all(&user_dir);
        load_user_themes_from(&user_dir, &mut out);
        out
    })
}

/// Index of the startup theme ("GitHub Dark").
pub fn default_index() -> usize {
    all()
        .iter()
        .position(|t| t.name == "GitHub Dark")
        .unwrap_or(0)
}

/// The embedded theme families only — no user-directory merge.
///
/// Test helper: assertions about the *shipped* assets go through this so they
/// hold on dev machines that have override themes in [`user_themes_dir`].
#[cfg(test)]
pub(crate) fn embedded_themes() -> Vec<Theme> {
    let mut out = Vec::new();
    for file in THEME_FILES {
        if let Some(data) = crate::assets::AppAssets::get(file) {
            parse_family(std::str::from_utf8(&data.data).unwrap_or(""), &mut out);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_themes() {
        let themes = all();
        // 9 GitHub themes in the single embedded family. Not an exact
        // equality: a machine with files in `user_themes_dir()` legitimately
        // has more, and override-by-name can replace embedded ones.
        assert!(
            themes.len() >= 9,
            "expected at least the 9 embedded themes, got {}",
            themes.len()
        );
        assert!(themes.iter().any(|t| t.name == "GitHub Dark"));
        assert!(themes.iter().any(|t| t.name == "GitHub Light"));
        assert_eq!(all()[default_index()].name, "GitHub Dark");
    }

    #[test]
    fn single_theme_file_is_wrapped_into_a_family() {
        let json = r##"{
            "name": "Test Single",
            "appearance": "light",
            "style": {
                "background": "#fefefeff",
                "editor.background": "#fefefeff",
                "editor.foreground": "#111111ff",
                "syntax": { "comment": { "color": "#888888ff" } }
            }
        }"##;
        let mut out = Vec::new();
        parse_theme_file(json, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].name, "Test Single");
        assert_eq!(out[0].appearance, "light");
        assert_eq!(out[0].colors.editor_bg, 0xfefefeff);
        // Family and single-theme layouts share the fallback pipeline, so
        // every derived token is filled in.
        assert!(out[0].colors.is_complete());
        let hl = out[0].highlight_theme();
        assert!(hl.style.syntax.style("comment").is_some());
    }

    #[test]
    fn malformed_theme_files_are_skipped() {
        let mut out = Vec::new();
        parse_theme_file("not json at all", &mut out);
        parse_theme_file("{\"name\": \"no style block\"}", &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn user_themes_override_embedded_by_name() {
        let dir = std::env::temp_dir().join(format!("ezicode-theme-tests-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Same name as an embedded theme: must replace it, not duplicate it.
        std::fs::write(
            dir.join("override.json"),
            r##"{
                "name": "GitHub Dark",
                "appearance": "dark",
                "style": {
                    "background": "#050505ff",
                    "editor.background": "#050505ff",
                    "editor.foreground": "#eeeeeeff"
                }
            }"##,
        )
        .unwrap();
        // A new name: must be appended.
        std::fs::write(
            dir.join("extra.json"),
            r##"{
                "name": "My Custom Theme",
                "appearance": "dark",
                "style": { "background": "#101010ff", "editor.background": "#101010ff" }
            }"##,
        )
        .unwrap();

        let mut themes: Vec<Theme> = all().to_vec();
        let before = themes.len();
        load_user_themes_from(&dir, &mut themes);
        assert_eq!(themes.len(), before + 1, "one override + one addition");
        let overridden = themes.iter().find(|t| t.name == "GitHub Dark").unwrap();
        assert_eq!(overridden.colors.background, 0x050505ff);
        assert_eq!(themes.iter().filter(|t| t.name == "GitHub Dark").count(), 1);
        assert!(themes.iter().any(|t| t.name == "My Custom Theme"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn github_dark_values_match_source_file() {
        let embedded = embedded_themes();
        let gd = embedded
            .iter()
            .find(|t| t.name == "GitHub Dark")
            .expect("GitHub Dark is embedded");
        assert_eq!(gd.appearance, "dark");
        assert_eq!(gd.colors.background, 0x0d1117ff);
        assert_eq!(gd.colors.panel, 0x010409ff);
        assert_eq!(gd.colors.editor_fg, 0xf0f6fcff);
    }

    #[test]
    fn every_theme_has_every_token() {
        for t in all() {
            assert!(
                t.colors.is_complete(),
                "theme '{}' is missing tokens",
                t.name
            );
        }
    }

    #[test]
    fn light_and_dark_appearances_exist_for_widget_sync() {
        assert!(all().iter().any(|t| t.appearance == "light"));
        assert!(all().iter().any(|t| t.appearance == "dark"));
    }

    #[test]
    fn highlighter_uses_zed_syntax_palette() {
        use gpui::{rgba, Hsla};
        let embedded = embedded_themes();
        let od = embedded
            .iter()
            .find(|t| t.name == "GitHub Dark")
            .expect("GitHub Dark is embedded");
        let hl = od.highlight_theme();
        // GitHub Dark comment token is #9198a1 in the source JSON.
        let comment = hl.style.syntax.style("comment").expect("comment style");
        assert_eq!(comment.color, Some(Hsla::from(rgba(0x9198a1ff))));
        let function = hl.style.syntax.style("function").expect("function style");
        assert_eq!(function.color, Some(Hsla::from(rgba(0xd2a8ffff))));
        assert!(hl.style.syntax.style("keyword").is_some());
        assert!(hl.style.syntax.style("comment.doc").is_some());
    }

    #[test]
    fn terminal_palette_matches_theme() {
        let embedded = embedded_themes();
        let gd = embedded
            .iter()
            .find(|t| t.name == "GitHub Dark")
            .expect("GitHub Dark is embedded");
        // GitHub Dark background is #010409 (terminal.background)
        let bg = gd.terminal_palette.background();
        assert_eq!(bg.a, 1.0);
        // Verify all themes — embedded and any installed user themes — have
        // valid terminal palettes.
        for t in all() {
            assert_eq!(t.terminal_palette.ansi_colors().len(), 16);
            assert_eq!(t.terminal_palette.extended_colors().len(), 256);
        }
    }
}
