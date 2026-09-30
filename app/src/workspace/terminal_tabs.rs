//! Zed-style terminal tab management: reordering by drag & drop, pinning,
//! inline renaming, read-only toggling and the bulk close operations behind
//! the tab context menu.
//!
//! Everything here operates on the two existing tab lists owned by
//! [`Workspace`] (`terminal_tabs` for the bottom dock, `terminal_right_tabs`
//! for the right dock). Tabs are moved by shuffling `Entity<Terminal>`
//! handles inside those vectors — the terminal views, PTYs and child
//! processes are never touched, so reordering can't restart or drop a
//! session.
//!
//! Invariant maintained throughout (borrowed from Zed's `Pane`): pinned tabs
//! form a prefix of the tab list, tracked by a plain `pinned_count` per dock.

use gpui::{AppContext, Context, Entity, Window};
use gpui_component::input::{InputEvent, InputState};

use super::{TerminalDragTarget, TerminalRenaming, Workspace};
use crate::terminal::{Terminal, TerminalDock, TerminalState, TerminalTabDrag};

/// Result of planning a same-dock tab move: where the tab lands after the
/// removal shifted everything, and the dock's pinned count afterwards.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SameDockMovePlan {
    pub(super) insert_ix: usize,
    pub(super) pinned_count: usize,
}

/// Plan moving the tab at `from` so it lands at insertion point `to`
/// (`0..=len`, as reported by the drop indicator). Returns `None` for
/// no-op drops: out-of-range sources and drops onto the dragged tab itself
/// (`to == from` inserts before itself, `to == from + 1` after itself).
///
/// Pin semantics follow Zed's `handle_tab_drop`:
/// * dropping inside the pinned prefix pins the tab,
/// * dropping past it unpins,
/// * dropping exactly on the boundary keeps the tab's current state
///   (so nudging the last pinned tab one slot right leaves it pinned).
pub(super) fn plan_same_dock_move(
    len: usize,
    pinned_count: usize,
    from: usize,
    to: usize,
) -> Option<SameDockMovePlan> {
    if from >= len {
        return None;
    }
    let to = to.min(len);
    if to == from || to == from + 1 {
        return None;
    }
    let was_pinned = from < pinned_count;
    // Pinned count of the list with the dragged tab removed.
    let pc = pinned_count - usize::from(was_pinned);
    let insert_ix = if from < to { to - 1 } else { to };
    let stays_pinned = insert_ix < pc || (insert_ix == pc && was_pinned);
    Some(SameDockMovePlan {
        insert_ix,
        pinned_count: pc + usize::from(stays_pinned),
    })
}

/// Where the active index lands after closing `closed` (sorted, deduped)
/// out of a list whose active index was `old_active` and whose new length is
/// `new_len`. Mirrors the pane behaviour everywhere: closing tabs before the
/// active one keeps the same terminal active; closing the active tab
/// activates its right neighbour, or the new last tab when it was rightmost.
pub(super) fn active_after_close(old_active: usize, closed: &[usize], new_len: usize) -> usize {
    if new_len == 0 {
        return 0;
    }
    let removed_before = closed.iter().filter(|&&ix| ix < old_active).count();
    (old_active - removed_before).min(new_len - 1)
}

impl Workspace {
    // ------------------------------------------------------------------
    // Dock plumbing
    // ------------------------------------------------------------------

    /// The one place that maps a dock to its tab list, active index and
    /// pinned count. Returning all three keeps the borrow checker happy when
    /// an operation needs to mutate them together.
    fn terminal_dock_parts(
        &mut self,
        dock: TerminalDock,
    ) -> (&mut Vec<Entity<Terminal>>, &mut usize, &mut usize) {
        match dock {
            TerminalDock::Bottom => (
                &mut self.terminal_tabs,
                &mut self.active_terminal,
                &mut self.terminal_pinned_count,
            ),
            TerminalDock::Right => (
                &mut self.terminal_right_tabs,
                &mut self.active_terminal_right,
                &mut self.terminal_right_pinned_count,
            ),
        }
    }

    fn terminal_dock_tabs(&self, dock: TerminalDock) -> &[Entity<Terminal>] {
        match dock {
            TerminalDock::Bottom => &self.terminal_tabs,
            TerminalDock::Right => &self.terminal_right_tabs,
        }
    }

    fn terminal_dock_pinned_count(&self, dock: TerminalDock) -> usize {
        match dock {
            TerminalDock::Bottom => self.terminal_pinned_count,
            TerminalDock::Right => self.terminal_right_pinned_count,
        }
    }

    pub(crate) fn terminal_tab_is_pinned(&self, dock: TerminalDock, index: usize) -> bool {
        index < self.terminal_dock_pinned_count(dock)
    }

    /// Resolve a terminal entity back to its current index in a dock. All
    /// context-menu actions go through this at click time, so they operate
    /// on the right tab even after the list was reordered or shrunk while
    /// the menu was open.
    pub(crate) fn terminal_index_for(
        &self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
    ) -> Option<usize> {
        self.terminal_dock_tabs(dock)
            .iter()
            .position(|tab| tab == terminal)
    }

    fn reveal_terminal_dock_active(&mut self, dock: TerminalDock) {
        match dock {
            TerminalDock::Bottom => {
                if !self.terminal_tabs.is_empty() {
                    self.terminal_tab_scroll
                        .scroll_to_item(self.active_terminal);
                }
            }
            TerminalDock::Right => {
                if !self.terminal_right_tabs.is_empty() {
                    self.terminal_right_tab_scroll
                        .scroll_to_item(self.active_terminal_right);
                }
            }
        }
    }

    fn focus_terminal_dock_active(
        &mut self,
        dock: TerminalDock,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let terminal = match dock {
            TerminalDock::Bottom => self.terminal_tabs.get(self.active_terminal).cloned(),
            TerminalDock::Right => self
                .terminal_right_tabs
                .get(self.active_terminal_right)
                .cloned(),
        };
        if let Some(terminal) = terminal {
            terminal.read(cx).focus_handle(cx).focus(window);
        }
    }

    // ------------------------------------------------------------------
    // Closing
    // ------------------------------------------------------------------

    /// Multi-close core shared by every close pathway (close button, middle
    /// click, keyboard shortcut, all context-menu variants). Removes the
    /// given tabs in one pass, then repairs the pinned prefix, the active
    /// index, the inline rename and the dock visibility.
    pub(crate) fn close_terminal_tabs_at(
        &mut self,
        dock: TerminalDock,
        mut indices: Vec<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        indices.sort_unstable();
        indices.dedup();

        let single_label = (indices.len() == 1).then(|| indices[0] + 1);

        {
            let (tabs, _, _) = self.terminal_dock_parts(dock);
            indices.retain(|&ix| ix < tabs.len());
        }
        if indices.is_empty() {
            return;
        }

        let (tabs, active, pinned) = self.terminal_dock_parts(dock);
        let old_active = *active;
        let mut removed_pinned = 0usize;
        for &ix in indices.iter().rev() {
            // Dropping the entity here is what ends the terminal: the view,
            // its reader thread and the PTY all hang off this handle.
            tabs.remove(ix);
            if ix < *pinned {
                removed_pinned += 1;
            }
        }
        *pinned -= removed_pinned;
        *active = active_after_close(old_active, &indices, tabs.len());
        let remaining = tabs.len();

        // A rename targeting one of the closed tabs would linger forever —
        // its terminal can never be found again — so drop it now.
        let rename_is_stale = self.terminal_renaming.as_ref().is_some_and(|renaming| {
            self.terminal_index_for(renaming.dock, &renaming.terminal)
                .is_none()
        });
        if rename_is_stale {
            self.terminal_renaming = None;
        }

        if remaining == 0 {
            match dock {
                TerminalDock::Bottom => {
                    self.show_terminal = false;
                    self.terminal_maximized = false;
                    self.status = "Terminal closed".into();
                }
                TerminalDock::Right => {
                    self.show_terminal_right = false;
                    self.status = "Right terminal panel closed".into();
                }
            }
            self.focus_active_editor_or_self(window, cx);
            cx.notify();
            return;
        }

        self.status = match (dock, single_label) {
            (TerminalDock::Bottom, Some(n)) => {
                format!("Terminal {n} closed — {remaining} terminal(s) remain")
            }
            (TerminalDock::Right, Some(n)) => {
                format!("Right terminal {n} closed — {remaining} terminal(s) remain")
            }
            (_, None) => format!(
                "Closed {} terminal tab(s) — {remaining} remain",
                indices.len()
            ),
        };
        self.reveal_terminal_dock_active(dock);
        self.focus_terminal_dock_active(dock, window, cx);
        cx.notify();
    }

    /// Middle click: close the tab, sparing pinned ones (Zed parity).
    pub(crate) fn middle_click_close_terminal(
        &mut self,
        dock: TerminalDock,
        index: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.terminal_tab_is_pinned(dock, index) {
            self.status = "Terminal tab is pinned — unpin it to close".into();
            cx.notify();
            return;
        }
        self.close_terminal_tabs_at(dock, vec![index], window, cx);
    }

    /// Context menu "Close": closes the clicked tab, pinned or not — using
    /// the explicit menu action is a deliberate enough gesture (Zed closes
    /// pinned tabs from the menu too, only shortcuts spare them).
    pub(crate) fn close_terminal_tab_for(
        &mut self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(ix) = self.terminal_index_for(dock, terminal) {
            self.close_terminal_tabs_at(dock, vec![ix], window, cx);
        }
    }

    /// "Close Others": everything except the clicked tab; pinned tabs stay.
    pub(crate) fn close_other_terminal_tabs_for(
        &mut self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(keep) = self.terminal_index_for(dock, terminal) else {
            return;
        };
        let pinned = self.terminal_dock_pinned_count(dock);
        let indices: Vec<usize> = (0..self.terminal_dock_tabs(dock).len())
            .filter(|&ix| ix != keep && ix >= pinned)
            .collect();
        self.close_terminal_tabs_at(dock, indices, window, cx);
        // Whatever survived, the clicked tab is what the user chose to keep,
        // so it becomes (or stays) the active one.
        if let Some(ix) = self.terminal_index_for(dock, terminal) {
            let (_, active, _) = self.terminal_dock_parts(dock);
            *active = ix;
            self.reveal_terminal_dock_active(dock);
            self.focus_terminal_dock_active(dock, window, cx);
            cx.notify();
        }
    }

    /// "Close Left": tabs before the clicked one; pinned tabs stay.
    pub(crate) fn close_terminal_tabs_left_for(
        &mut self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ix) = self.terminal_index_for(dock, terminal) else {
            return;
        };
        let pinned = self.terminal_dock_pinned_count(dock);
        let indices: Vec<usize> = (pinned..ix).collect();
        self.close_terminal_tabs_at(dock, indices, window, cx);
    }

    /// "Close Right": tabs after the clicked one; pinned tabs stay (they
    /// can't be to the right of an unpinned tab, but the clicked tab itself
    /// may be pinned).
    pub(crate) fn close_terminal_tabs_right_for(
        &mut self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(ix) = self.terminal_index_for(dock, terminal) else {
            return;
        };
        let pinned = self.terminal_dock_pinned_count(dock);
        let indices: Vec<usize> = (ix + 1..self.terminal_dock_tabs(dock).len())
            .filter(|&i| i >= pinned)
            .collect();
        self.close_terminal_tabs_at(dock, indices, window, cx);
    }

    /// "Close Clean": terminals whose process already exited (the terminal
    /// equivalent of Zed's not-dirty items); pinned tabs stay.
    pub(crate) fn close_clean_terminal_tabs_for(
        &mut self,
        dock: TerminalDock,
        _terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pinned = self.terminal_dock_pinned_count(dock);
        let indices: Vec<usize> = self
            .terminal_dock_tabs(dock)
            .iter()
            .enumerate()
            .filter(|(ix, tab)| *ix >= pinned && tab.read(cx).state != TerminalState::Running)
            .map(|(ix, _)| ix)
            .collect();
        if indices.is_empty() {
            self.status = "No clean terminal tabs to close".into();
            cx.notify();
            return;
        }
        self.close_terminal_tabs_at(dock, indices, window, cx);
    }

    /// "Close All": every tab in the dock; pinned tabs stay (Zed parity).
    pub(crate) fn close_all_terminal_tabs_for(
        &mut self,
        dock: TerminalDock,
        _terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let pinned = self.terminal_dock_pinned_count(dock);
        let indices: Vec<usize> = (pinned..self.terminal_dock_tabs(dock).len()).collect();
        if indices.is_empty() {
            self.status = "All terminal tabs are pinned".into();
            cx.notify();
            return;
        }
        self.close_terminal_tabs_at(dock, indices, window, cx);
    }

    // ------------------------------------------------------------------
    // Pinning
    // ------------------------------------------------------------------

    /// Pin ⇄ unpin the tab at `index`. Pinning moves the tab to the end of
    /// the pinned prefix; unpinning parks it right after the prefix — the
    /// same shuffle Zed's `pin_tab_at` / `unpin_tab_at` perform.
    pub(crate) fn toggle_terminal_pin_at(
        &mut self,
        dock: TerminalDock,
        index: usize,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (tabs, active, pinned) = self.terminal_dock_parts(dock);
        if index >= tabs.len() {
            return;
        }
        let active_entity = tabs.get(*active).cloned();
        let now_pinned = if index < *pinned {
            *pinned -= 1;
            let target = *pinned;
            let tab = tabs.remove(index);
            tabs.insert(target, tab);
            false
        } else {
            let target = *pinned;
            let tab = tabs.remove(index);
            tabs.insert(target, tab);
            *pinned += 1;
            true
        };
        // The move may have shifted the active tab; follow it by entity so
        // the selection never lands on a different terminal.
        if let Some(active_entity) = active_entity {
            if let Some(pos) = tabs.iter().position(|tab| tab == &active_entity) {
                *active = pos;
            }
        }
        self.status = if now_pinned {
            "Terminal tab pinned".into()
        } else {
            "Terminal tab unpinned".into()
        };
        self.reveal_terminal_dock_active(dock);
        cx.notify();
    }

    pub(crate) fn toggle_terminal_pin_for(
        &mut self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(ix) = self.terminal_index_for(dock, terminal) {
            self.toggle_terminal_pin_at(dock, ix, window, cx);
        }
    }

    // ------------------------------------------------------------------
    // Read-only
    // ------------------------------------------------------------------

    pub(crate) fn toggle_terminal_read_only_for(
        &mut self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
        cx: &mut Context<Self>,
    ) {
        if self.terminal_index_for(dock, terminal).is_none() {
            return;
        }
        let mut read_only = false;
        terminal.update(cx, |term, cx| {
            read_only = !term.is_read_only();
            term.set_read_only(read_only, cx);
        });
        self.status = if read_only {
            "Terminal tab is now read-only".into()
        } else {
            "Terminal tab is editable again".into()
        };
        cx.notify();
    }

    // ------------------------------------------------------------------
    // Renaming
    // ------------------------------------------------------------------

    /// Swap the tab's label for an inline input (Enter commits, Escape or
    /// clicking elsewhere cancels), mirroring the explorer's inline rename.
    pub(crate) fn start_terminal_rename_for(
        &mut self,
        dock: TerminalDock,
        terminal: &Entity<Terminal>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.terminal_index_for(dock, terminal).is_none() {
            return;
        }
        // Only one rename at a time; starting a new one abandons the old.
        self.terminal_renaming = None;

        let current = terminal.read(cx).tab_label().to_string();
        let input = cx.new(|cx| {
            let mut state = InputState::new(window, cx).default_value(current);
            state.select_all_text(cx);
            state.focus(window, cx);
            state
        });
        cx.subscribe_in(
            &input,
            window,
            |this, _input, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } => this.confirm_terminal_rename(window, cx),
                InputEvent::Blur => this.cancel_terminal_rename(cx),
                _ => {}
            },
        )
        .detach();
        self.terminal_renaming = Some(TerminalRenaming {
            dock,
            terminal: terminal.clone(),
            input,
        });
        cx.notify();
    }

    pub(crate) fn confirm_terminal_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(renaming) = self.terminal_renaming.take() else {
            return;
        };
        let value = renaming.input.read(cx).value().trim().to_string();
        let cleared = value.is_empty();
        renaming.terminal.update(cx, |terminal, cx| {
            terminal.set_custom_title((!cleared).then_some(value), cx);
        });
        self.status = if cleared {
            // An empty name falls back to the automatic title, like Zed.
            "Terminal tab name reset".into()
        } else {
            "Terminal tab renamed".into()
        };
        // Hand focus back to the terminal that was renamed.
        renaming.terminal.read(cx).focus_handle(cx).focus(window);
        cx.notify();
    }

    pub(crate) fn cancel_terminal_rename(&mut self, cx: &mut Context<Self>) {
        if self.terminal_renaming.take().is_some() {
            cx.notify();
        }
    }

    // ------------------------------------------------------------------
    // Drag & drop
    // ------------------------------------------------------------------

    /// Called from `on_drag_move` while a tab drag hovers a drop position.
    /// Only notifies when the target actually changes, so dragging across a
    /// tab doesn't re-render the workspace on every mouse move.
    pub(crate) fn set_terminal_drag_target(
        &mut self,
        dock: TerminalDock,
        insert_ix: usize,
        owner_ix: usize,
        cx: &mut Context<Self>,
    ) {
        let next = Some(TerminalDragTarget {
            dock,
            insert_ix,
            owner_ix,
        });
        if self.terminal_drag_target != next {
            self.terminal_drag_target = next;
            cx.notify();
        }
    }

    /// Clear the drop indicator, but only if `owner_ix` set it — two tabs
    /// share each boundary index, and the one the pointer just left must not
    /// wipe the indicator its neighbour just placed.
    pub(crate) fn clear_terminal_drag_target_from(
        &mut self,
        dock: TerminalDock,
        owner_ix: usize,
        cx: &mut Context<Self>,
    ) {
        if let Some(target) = self.terminal_drag_target {
            if target.dock == dock && target.owner_ix == owner_ix {
                self.terminal_drag_target = None;
                cx.notify();
            }
        }
    }

    /// The drop indicator insert position for a dock, for rendering.
    pub(crate) fn terminal_drag_target_for(&self, dock: TerminalDock) -> Option<usize> {
        self.terminal_drag_target
            .filter(|target| target.dock == dock)
            .map(|target| target.insert_ix)
    }

    /// Finish a tab drag. `fallback_insert` is used when no live indicator
    /// exists for the dock (e.g. the drop landed without a preceding move
    /// event); otherwise the indicator the user saw wins.
    pub(crate) fn drop_terminal_tab(
        &mut self,
        drag: &TerminalTabDrag,
        target_dock: TerminalDock,
        fallback_insert: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let insert_ix = match self.terminal_drag_target {
            Some(target) if target.dock == target_dock => target.insert_ix,
            _ => fallback_insert,
        };
        if self.terminal_drag_target.take().is_some() {
            cx.notify();
        }

        // Resolve the dragged terminal by entity: its index may have shifted
        // (or it may have been closed) since the drag started.
        let source = [TerminalDock::Bottom, TerminalDock::Right]
            .into_iter()
            .find_map(|dock| {
                self.terminal_index_for(dock, &drag.terminal)
                    .map(|ix| (dock, ix))
            });
        let Some((source_dock, from_ix)) = source else {
            return;
        };

        if source_dock == target_dock {
            self.move_terminal_tab_within(source_dock, from_ix, insert_ix, window, cx);
        } else {
            self.move_terminal_tab_across(source_dock, from_ix, target_dock, insert_ix, window, cx);
        }
    }

    /// Reorder within one dock. The entity is moved inside the vector —
    /// nothing about the terminal itself changes, so the running process,
    /// scrollback and selection all survive untouched.
    fn move_terminal_tab_within(
        &mut self,
        dock: TerminalDock,
        from: usize,
        to: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let (tabs, active, pinned) = self.terminal_dock_parts(dock);
        let Some(plan) = plan_same_dock_move(tabs.len(), *pinned, from, to) else {
            return;
        };
        let tab = tabs.remove(from);
        tabs.insert(plan.insert_ix, tab);
        *pinned = plan.pinned_count;
        // Zed activates a dropped tab; since the dragged entity now lives at
        // the planned index, this also keeps the selection on the same
        // terminal the user was holding.
        *active = plan.insert_ix;

        self.status = "Terminal tab moved".into();
        self.reveal_terminal_dock_active(dock);
        self.focus_terminal_dock_active(dock, window, cx);
        cx.notify();
    }

    /// Move a tab between the two docks (bottom ⇄ right), preserving the
    /// session. Mirrors dragging a tab into another pane in Zed: the tab is
    /// activated and focused in its new home, and an emptied source dock
    /// hides itself.
    fn move_terminal_tab_across(
        &mut self,
        source_dock: TerminalDock,
        from: usize,
        target_dock: TerminalDock,
        to: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Detach from the source dock.
        let (tabs, active, pinned) = self.terminal_dock_parts(source_dock);
        if from >= tabs.len() {
            return;
        }
        let was_pinned = from < *pinned;
        let old_active = *active;
        let tab = tabs.remove(from);
        if was_pinned {
            *pinned -= 1;
        }
        *active = active_after_close(old_active, &[from], tabs.len());
        let source_now_empty = tabs.is_empty();

        // Attach to the target dock.
        let (tabs, active, pinned) = self.terminal_dock_parts(target_dock);
        let to = to.min(tabs.len());
        let stays_pinned = to < *pinned || (to == *pinned && was_pinned);
        tabs.insert(to, tab);
        if stays_pinned {
            *pinned += 1;
        }
        *active = to;

        if source_now_empty {
            match source_dock {
                TerminalDock::Bottom => {
                    self.show_terminal = false;
                    self.terminal_maximized = false;
                }
                TerminalDock::Right => self.show_terminal_right = false,
            }
        }
        match target_dock {
            TerminalDock::Bottom => self.show_terminal = true,
            TerminalDock::Right => self.show_terminal_right = true,
        }

        self.status = match target_dock {
            TerminalDock::Bottom => "Terminal tab moved to the bottom panel".into(),
            TerminalDock::Right => "Terminal tab moved to the right panel".into(),
        };
        self.reveal_terminal_dock_active(target_dock);
        self.focus_terminal_dock_active(target_dock, window, cx);
        cx.notify();
    }

    // ------------------------------------------------------------------
    // Keyboard entry points (act on the focused dock's active tab)
    // ------------------------------------------------------------------

    fn focused_terminal_target(
        &self,
        window: &Window,
        cx: &Context<Self>,
    ) -> Option<(TerminalDock, Entity<Terminal>)> {
        self.focused_or_active_terminal(window, cx)
            .map(|(dock, _, terminal)| (dock, terminal))
    }

    pub(crate) fn close_other_terminals_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.close_other_terminal_tabs_for(dock, &terminal, window, cx);
        }
    }

    pub(crate) fn close_terminals_left_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.close_terminal_tabs_left_for(dock, &terminal, window, cx);
        }
    }

    pub(crate) fn close_terminals_right_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.close_terminal_tabs_right_for(dock, &terminal, window, cx);
        }
    }

    pub(crate) fn close_clean_terminals_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.close_clean_terminal_tabs_for(dock, &terminal, window, cx);
        }
    }

    pub(crate) fn close_all_terminals_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.close_all_terminal_tabs_for(dock, &terminal, window, cx);
        }
    }

    pub(crate) fn toggle_terminal_pin_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.toggle_terminal_pin_for(dock, &terminal, window, cx);
        }
    }

    pub(crate) fn rename_terminal_focused(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.start_terminal_rename_for(dock, &terminal, window, cx);
        }
    }

    pub(crate) fn toggle_terminal_read_only_focused(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some((dock, terminal)) = self.focused_terminal_target(window, cx) {
            self.toggle_terminal_read_only_for(dock, &terminal, cx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- plan_same_dock_move -----------------------------------------

    #[test]
    fn move_to_front_from_end() {
        // [a b c d] drag d to the very front.
        let plan = plan_same_dock_move(4, 0, 3, 0).unwrap();
        assert_eq!(plan.insert_ix, 0);
        assert_eq!(plan.pinned_count, 0);
    }

    #[test]
    fn move_first_to_end() {
        // [a b c d] drag a past the last tab (insertion index == len).
        let plan = plan_same_dock_move(4, 0, 0, 4).unwrap();
        assert_eq!(plan.insert_ix, 3);
        assert_eq!(plan.pinned_count, 0);
    }

    #[test]
    fn move_middle_right() {
        // [a b c d] drag b so it lands after c.
        let plan = plan_same_dock_move(4, 0, 1, 3).unwrap();
        assert_eq!(plan.insert_ix, 2);
    }

    #[test]
    fn drop_on_self_is_noop() {
        // Both edges of the dragged tab are no-ops.
        assert!(plan_same_dock_move(4, 0, 2, 2).is_none());
        assert!(plan_same_dock_move(4, 0, 2, 3).is_none());
    }

    #[test]
    fn single_tab_cannot_move() {
        assert!(plan_same_dock_move(1, 0, 0, 0).is_none());
        assert!(plan_same_dock_move(1, 0, 0, 1).is_none());
    }

    #[test]
    fn out_of_range_source_is_noop() {
        assert!(plan_same_dock_move(2, 0, 5, 0).is_none());
    }

    #[test]
    fn insertion_index_is_clamped() {
        let plan = plan_same_dock_move(3, 0, 0, 99).unwrap();
        assert_eq!(plan.insert_ix, 2);
    }

    #[test]
    fn dragging_unpinned_into_pinned_region_pins_it() {
        // [P p | c d] drag d between the two pinned tabs.
        let plan = plan_same_dock_move(4, 2, 3, 1).unwrap();
        assert_eq!(plan.insert_ix, 1);
        assert_eq!(plan.pinned_count, 3);
    }

    #[test]
    fn dragging_pinned_out_of_region_unpins_it() {
        // [P p | c d] drag the first pinned tab past d.
        let plan = plan_same_dock_move(4, 2, 0, 4).unwrap();
        assert_eq!(plan.insert_ix, 3);
        assert_eq!(plan.pinned_count, 1);
    }

    #[test]
    fn dragging_pinned_tab_past_unpinned_tab_unpins_it() {
        // [p | b] dragging p past b clearly leaves the pinned region.
        let plan = plan_same_dock_move(2, 1, 0, 2).unwrap();
        assert_eq!(plan.insert_ix, 1);
        assert_eq!(plan.pinned_count, 0);
    }

    #[test]
    fn pinned_tab_dropped_on_boundary_stays_pinned() {
        // Zed regression test: [p1 p2 | b] dragging p1 onto the boundary
        // (right after p2) keeps it pinned.
        let plan = plan_same_dock_move(3, 2, 0, 2).unwrap();
        assert_eq!(plan.insert_ix, 1);
        assert_eq!(plan.pinned_count, 2);
    }

    #[test]
    fn unpinned_tab_dropped_on_boundary_stays_unpinned() {
        // [p | b c] drag c to just after the pinned prefix.
        let plan = plan_same_dock_move(3, 1, 2, 1).unwrap();
        assert_eq!(plan.insert_ix, 1);
        assert_eq!(plan.pinned_count, 1);
    }

    // ---- active_after_close --------------------------------------------

    #[test]
    fn closing_before_active_keeps_the_same_terminal_active() {
        assert_eq!(active_after_close(2, &[0], 3), 1);
    }

    #[test]
    fn closing_after_active_leaves_index_alone() {
        assert_eq!(active_after_close(0, &[2], 3), 0);
    }

    #[test]
    fn closing_the_active_tab_activates_its_right_neighbour() {
        // [a b* c] closing b: index 1 now names c.
        assert_eq!(active_after_close(1, &[1], 2), 1);
    }

    #[test]
    fn closing_the_last_active_tab_activates_the_new_last() {
        assert_eq!(active_after_close(2, &[2], 2), 1);
    }

    #[test]
    fn closing_everything_resets_to_zero() {
        assert_eq!(active_after_close(1, &[0, 1, 2], 0), 0);
    }

    #[test]
    fn close_others_keeps_the_survivor_reachable() {
        // 5 tabs, active 4, closing 0..4 → one survivor at index 0.
        assert_eq!(active_after_close(4, &[0, 1, 2, 3], 1), 0);
    }
}
