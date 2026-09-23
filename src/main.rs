use std::collections::BTreeMap;
use zellij_tile::prelude::*;

fn mode_color(mode: InputMode) -> u8 {
    match mode {
        InputMode::Locked => 9,
        InputMode::Prompt | InputMode::Tmux => 10,
        InputMode::Resize | InputMode::Move => 11,
        InputMode::Pane | InputMode::Tab | InputMode::RenamePane | InputMode::RenameTab => 12,
        InputMode::Scroll | InputMode::EnterSearch | InputMode::Search => 13,
        InputMode::Session => 14,
        InputMode::Normal => 201,
    }
}

#[derive(Default)]
struct State {
    frame_highlight: u8,
    frame_selected: u8,
}

impl State {
    fn update_pane_frame_colors(&mut self, frame_selected: u8, frame_highlight: u8) {
        if self.frame_selected == frame_selected && self.frame_highlight == frame_highlight {
            return;
        }
        self.frame_selected = frame_selected;
        self.frame_highlight = frame_highlight;
        set_pane_frame_colors(
            PaletteColor::EightBit(self.frame_selected),
            PaletteColor::EightBit(self.frame_highlight),
        );
    }

    fn handle_normal_mode(&mut self, new_selected: u8) {
        self.update_pane_frame_colors(new_selected, new_selected);
    }

    fn handle_locked_mode(&mut self, new_selected: u8) {
        self.update_pane_frame_colors(new_selected, new_selected);
    }

    fn handle_highlight_mode(&mut self, new_highlight: u8) {
        self.update_pane_frame_colors(self.frame_selected, new_highlight);
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        subscribe(&[EventType::ModeUpdate]);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
    }
    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::ModeUpdate(mode_info) => {
                match mode_info.mode {
                    InputMode::Normal => self.handle_normal_mode(mode_color(mode_info.mode)),
                    InputMode::Locked => self.handle_locked_mode(mode_color(mode_info.mode)),
                    _ => self.handle_highlight_mode(mode_color(mode_info.mode)),
                }
                false
            }
            _ => false,
        }
    }
    fn pipe(&mut self, _pipe_message: PipeMessage) -> bool {
        false
    }
    fn render(&mut self, _rows: usize, _cols: usize) {}
}

register_plugin!(State);
