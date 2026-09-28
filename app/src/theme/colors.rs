#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    pub background: u32,
    pub surface: u32,
    pub elevated_surface: u32,
    pub element_bg: u32,
    pub element_hover: u32,
    pub element_active: u32,
    pub element_selected: u32,
    pub ghost_hover: u32,
    pub ghost_active: u32,
    pub border: u32,
    pub border_variant: u32,
    pub border_focused: u32,
    pub text: u32,
    pub text_muted: u32,
    pub text_accent: u32,
    pub icon: u32,
    pub icon_muted: u32,
    pub icon_accent: u32,
    pub title_bar: u32,
    pub status_bar: u32,
    pub panel: u32,
    pub toolbar: u32,
    pub tab_bar: u32,
    pub tab_active_bg: u32,
    pub tab_inactive_bg: u32,
    pub tab_active_fg: u32,
    pub tab_inactive_fg: u32,
    pub editor_bg: u32,
    pub editor_fg: u32,
    pub terminal_bg: u32,
    pub vc_added: u32,
    pub vc_modified: u32,
    pub vc_deleted: u32,
}

impl Colors {
    pub(crate) const MISSING: u32 = 0xFF00FF;

    pub(crate) fn all_missing() -> Self {
        Self {
            background: Self::MISSING,
            surface: Self::MISSING,
            elevated_surface: Self::MISSING,
            element_bg: Self::MISSING,
            element_hover: Self::MISSING,
            element_active: Self::MISSING,
            element_selected: Self::MISSING,
            ghost_hover: Self::MISSING,
            ghost_active: Self::MISSING,
            border: Self::MISSING,
            border_variant: Self::MISSING,
            border_focused: Self::MISSING,
            text: Self::MISSING,
            text_muted: Self::MISSING,
            text_accent: Self::MISSING,
            icon: Self::MISSING,
            icon_muted: Self::MISSING,
            icon_accent: Self::MISSING,
            title_bar: Self::MISSING,
            status_bar: Self::MISSING,
            panel: Self::MISSING,
            toolbar: Self::MISSING,
            tab_bar: Self::MISSING,
            tab_active_bg: Self::MISSING,
            tab_inactive_bg: Self::MISSING,
            tab_active_fg: Self::MISSING,
            tab_inactive_fg: Self::MISSING,
            editor_bg: Self::MISSING,
            editor_fg: Self::MISSING,
            terminal_bg: Self::MISSING,
            vc_added: Self::MISSING,
            vc_modified: Self::MISSING,
            vc_deleted: Self::MISSING,
        }
    }

    #[cfg(test)]
    fn values(&self) -> [u32; 33] {
        [
            self.background,
            self.surface,
            self.elevated_surface,
            self.element_bg,
            self.element_hover,
            self.element_active,
            self.element_selected,
            self.ghost_hover,
            self.ghost_active,
            self.border,
            self.border_variant,
            self.border_focused,
            self.text,
            self.text_muted,
            self.text_accent,
            self.icon,
            self.icon_muted,
            self.icon_accent,
            self.title_bar,
            self.status_bar,
            self.panel,
            self.toolbar,
            self.tab_bar,
            self.tab_active_bg,
            self.tab_inactive_bg,
            self.tab_active_fg,
            self.tab_inactive_fg,
            self.editor_bg,
            self.editor_fg,
            self.terminal_bg,
            self.vc_added,
            self.vc_modified,
            self.vc_deleted,
        ]
    }

    pub(crate) fn get_mut(&mut self, field: &str) -> Option<&mut u32> {
        match field {
            "background" => Some(&mut self.background),
            "surface" => Some(&mut self.surface),
            "elevated_surface" => Some(&mut self.elevated_surface),
            "element_bg" => Some(&mut self.element_bg),
            "element_hover" => Some(&mut self.element_hover),
            "element_active" => Some(&mut self.element_active),
            "element_selected" => Some(&mut self.element_selected),
            "ghost_hover" => Some(&mut self.ghost_hover),
            "ghost_active" => Some(&mut self.ghost_active),
            "border" => Some(&mut self.border),
            "border_variant" => Some(&mut self.border_variant),
            "border_focused" => Some(&mut self.border_focused),
            "text" => Some(&mut self.text),
            "text_muted" => Some(&mut self.text_muted),
            "text_accent" => Some(&mut self.text_accent),
            "icon" => Some(&mut self.icon),
            "icon_muted" => Some(&mut self.icon_muted),
            "icon_accent" => Some(&mut self.icon_accent),
            "title_bar" => Some(&mut self.title_bar),
            "status_bar" => Some(&mut self.status_bar),
            "panel" => Some(&mut self.panel),
            "toolbar" => Some(&mut self.toolbar),
            "tab_bar" => Some(&mut self.tab_bar),
            "tab_active_bg" => Some(&mut self.tab_active_bg),
            "tab_inactive_bg" => Some(&mut self.tab_inactive_bg),
            "tab_active_fg" => Some(&mut self.tab_active_fg),
            "tab_inactive_fg" => Some(&mut self.tab_inactive_fg),
            "editor_bg" => Some(&mut self.editor_bg),
            "editor_fg" => Some(&mut self.editor_fg),
            "terminal_bg" => Some(&mut self.terminal_bg),
            "vc_added" => Some(&mut self.vc_added),
            "vc_modified" => Some(&mut self.vc_modified),
            "vc_deleted" => Some(&mut self.vc_deleted),
            _ => None,
        }
    }

    /// Immutable twin of [`Colors::get_mut`], used to read base-palette
    /// values during refinement.
    pub(crate) fn get(&self, field: &str) -> Option<u32> {
        match field {
            "background" => Some(self.background),
            "surface" => Some(self.surface),
            "elevated_surface" => Some(self.elevated_surface),
            "element_bg" => Some(self.element_bg),
            "element_hover" => Some(self.element_hover),
            "element_active" => Some(self.element_active),
            "element_selected" => Some(self.element_selected),
            "ghost_hover" => Some(self.ghost_hover),
            "ghost_active" => Some(self.ghost_active),
            "border" => Some(self.border),
            "border_variant" => Some(self.border_variant),
            "border_focused" => Some(self.border_focused),
            "text" => Some(self.text),
            "text_muted" => Some(self.text_muted),
            "text_accent" => Some(self.text_accent),
            "icon" => Some(self.icon),
            "icon_muted" => Some(self.icon_muted),
            "icon_accent" => Some(self.icon_accent),
            "title_bar" => Some(self.title_bar),
            "status_bar" => Some(self.status_bar),
            "panel" => Some(self.panel),
            "toolbar" => Some(self.toolbar),
            "tab_bar" => Some(self.tab_bar),
            "tab_active_bg" => Some(self.tab_active_bg),
            "tab_inactive_bg" => Some(self.tab_inactive_bg),
            "tab_active_fg" => Some(self.tab_active_fg),
            "tab_inactive_fg" => Some(self.tab_inactive_fg),
            "editor_bg" => Some(self.editor_bg),
            "editor_fg" => Some(self.editor_fg),
            "terminal_bg" => Some(self.terminal_bg),
            "vc_added" => Some(self.vc_added),
            "vc_modified" => Some(self.vc_modified),
            "vc_deleted" => Some(self.vc_deleted),
            _ => None,
        }
    }

    /// Fill every still-missing token from the light or dark base palette,
    /// selected by the theme's `appearance`.
    ///
    /// This mirrors Zed's `refine_theme_style`, which completes a user theme
    /// against the default colors for its appearance: a theme file only has
    /// to define the tokens it cares about, and the rest still render
    /// sensibly (light text on a dark base, dark text on a light base)
    /// instead of leaking the magenta `MISSING` sentinel into the UI.
    ///
    /// Runs *after* the contextual derivations in `parse_family` (tab bar
    /// from toolbar/panel/surface, active tab from the editor, …) so those
    /// more specific fallbacks always win over the base palette.
    pub(crate) fn refine_from_base(&mut self, appearance: &str) {
        let base = if appearance == "light" {
            &BASE_LIGHT
        } else {
            &BASE_DARK
        };
        for (field, _) in KEY_MAP {
            if let Some(slot) = self.get_mut(field) {
                if *slot == Self::MISSING {
                    *slot = base.get(field).unwrap_or(Self::MISSING);
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn is_complete(&self) -> bool {
        !self.values().contains(&Self::MISSING)
    }
}

pub(crate) const KEY_MAP: &[(&str, &str)] = &[
    ("background", "background"),
    ("surface", "surface.background"),
    ("elevated_surface", "elevated_surface.background"),
    ("element_bg", "element.background"),
    ("element_hover", "element.hover"),
    ("element_active", "element.active"),
    ("element_selected", "element.selected"),
    ("ghost_hover", "ghost_element.hover"),
    ("ghost_active", "ghost_element.active"),
    ("border", "border"),
    ("border_variant", "border.variant"),
    ("border_focused", "border.focused"),
    ("text", "text"),
    ("text_muted", "text.muted"),
    ("text_accent", "text.accent"),
    ("icon", "icon"),
    ("icon_muted", "icon.muted"),
    ("icon_accent", "icon.accent"),
    ("title_bar", "title_bar.background"),
    ("status_bar", "status_bar.background"),
    ("panel", "panel.background"),
    ("toolbar", "toolbar.background"),
    ("tab_bar", "tab_bar.background"),
    ("tab_active_bg", "tab.active_background"),
    ("tab_inactive_bg", "tab.inactive_background"),
    ("tab_active_fg", "tab.active_foreground"),
    ("tab_inactive_fg", "tab.inactive_foreground"),
    ("editor_bg", "editor.background"),
    ("editor_fg", "editor.foreground"),
    ("terminal_bg", "terminal.background"),
    ("vc_added", "version_control.added"),
    ("vc_modified", "version_control.modified"),
    ("vc_deleted", "version_control.deleted"),
];

pub(crate) const FALLBACKS: &[(&str, u32)] = &[
    ("vc_added", 0x27a657ff),
    ("vc_modified", 0xd3b020ff),
    ("vc_deleted", 0xe06c76ff),
];

/// Base dark palette used by [`Colors::refine_from_base`]: the exact final
/// token set of the shipped "GitHub Dark" theme (file values plus its
/// derivations), so refining that theme is a no-op and any incomplete dark
/// user theme degrades to the editor's default look.
pub(crate) const BASE_DARK: Colors = Colors {
    background: 0x0d1117ff,
    surface: 0x010409ff,
    elevated_surface: 0x010409ff,
    element_bg: 0x656c7633,
    element_hover: 0x656c7633,
    element_active: 0x656c7633,
    element_selected: 0x656c7633,
    ghost_hover: 0x656c7633,
    ghost_active: 0x656c7633,
    border: 0x3d444dff,
    border_variant: 0x3d444db3,
    border_focused: 0x1f6febff,
    text: 0xf0f6fcff,
    text_muted: 0xf0f6fcff,
    text_accent: 0x4493f8ff,
    icon: 0xf0f6fcff,
    icon_muted: 0x9198a1ff,
    icon_accent: 0x4493f8ff,
    title_bar: 0x010409ff,
    status_bar: 0x010409ff,
    panel: 0x010409ff,
    toolbar: 0x0d1117ff,
    tab_bar: 0x010409ff,
    tab_active_bg: 0x0d1117ff,
    tab_inactive_bg: 0x010409ff,
    tab_active_fg: 0xf0f6fcff,
    tab_inactive_fg: 0xf0f6fcff,
    editor_bg: 0x0d1117ff,
    editor_fg: 0xf0f6fcff,
    terminal_bg: 0x010409ff,
    vc_added: 0x27a657ff,
    vc_modified: 0xd3b020ff,
    vc_deleted: 0xe06c76ff,
};

/// Base light palette used by [`Colors::refine_from_base`]: the exact final
/// token set of the shipped "GitHub Light" theme.
pub(crate) const BASE_LIGHT: Colors = Colors {
    background: 0xffffffff,
    surface: 0xf6f8faff,
    elevated_surface: 0xffffffff,
    element_bg: 0x818b981f,
    element_hover: 0x818b981f,
    element_active: 0x818b981f,
    element_selected: 0x818b981f,
    ghost_hover: 0x818b981f,
    ghost_active: 0x818b981f,
    border: 0xd1d9e0ff,
    border_variant: 0xd1d9e0b3,
    border_focused: 0x0969daff,
    text: 0x1f2328ff,
    text_muted: 0x1f2328ff,
    text_accent: 0x0969daff,
    icon: 0x1f2328ff,
    icon_muted: 0x59636eff,
    icon_accent: 0x0969daff,
    title_bar: 0xf6f8faff,
    status_bar: 0xf6f8faff,
    panel: 0xf6f8faff,
    toolbar: 0xffffffff,
    tab_bar: 0xf6f8faff,
    tab_active_bg: 0xffffffff,
    tab_inactive_bg: 0xf6f8faff,
    tab_active_fg: 0x1f2328ff,
    tab_inactive_fg: 0x1f2328ff,
    editor_bg: 0xffffffff,
    editor_fg: 0x1f2328ff,
    terminal_bg: 0xf6f8faff,
    vc_added: 0x27a657ff,
    vc_modified: 0xd3b020ff,
    vc_deleted: 0xe06c76ff,
};

pub(crate) fn parse_hex(s: &str) -> Option<u32> {
    let hex = s.strip_prefix('#')?;
    if hex.len() == 6 {
        u32::from_str_radix(&format!("{hex}ff"), 16).ok()
    } else {
        u32::from_str_radix(hex, 16).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_with_and_without_alpha() {
        assert_eq!(parse_hex("#282c34"), Some(0x282c34ff));
        assert_eq!(parse_hex("#83899480"), Some(0x83899480));
        assert_eq!(parse_hex("nope"), None);
    }

    #[test]
    fn base_palettes_match_the_shipped_github_themes() {
        // `refine_from_base` must be a no-op for the themes the base
        // constants were extracted from — if a shipped GitHub theme changes,
        // the constants must be regenerated with it.
        let themes = crate::theme::embedded_themes();
        let dark = themes
            .iter()
            .find(|t| t.name == "GitHub Dark")
            .expect("GitHub Dark ships");
        assert_eq!(dark.colors, BASE_DARK);
        let light = themes
            .iter()
            .find(|t| t.name == "GitHub Light")
            .expect("GitHub Light ships");
        assert_eq!(light.colors, BASE_LIGHT);
    }

    #[test]
    fn refine_fills_every_token_and_keeps_appearance_contrast() {
        let mut minimal = Colors::all_missing();
        minimal.background = 0x050505ff;
        minimal.editor_bg = 0x050505ff;
        minimal.editor_fg = 0xeeeeeeff;
        minimal.refine_from_base("dark");
        assert!(minimal.is_complete());
        // Base dark text on the theme's own dark background: readable.
        assert_eq!(minimal.text, BASE_DARK.text);

        let mut light = Colors::all_missing();
        light.background = 0xfefefeff;
        light.editor_bg = 0xfefefeff;
        light.editor_fg = 0x111111ff;
        light.refine_from_base("light");
        assert!(light.is_complete());
        assert_eq!(light.text, BASE_LIGHT.text);
    }
}
