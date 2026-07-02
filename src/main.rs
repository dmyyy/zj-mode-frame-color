use regex::Regex;
use std::collections::BTreeMap;
use zellij_tile::prelude::*;

const CONFIG_PATH: &'static str = "/host/.config/zellij/config.kdl";

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
    re_frame_highlight: Option<Regex>,
    re_frame_selected: Option<Regex>,
}

impl State {
    fn frame_highlight_regex(&self) -> &Regex {
        self.re_frame_highlight.as_ref().unwrap()
    }

    fn frame_selected_regex(&self) -> &Regex {
        self.re_frame_selected.as_ref().unwrap()
    }

    fn write_updated_config(
        &self,
        frame_selected: Option<u8>,
        frame_highlight: Option<u8>,
    ) -> std::io::Result<()> {
        let config = std::fs::read_to_string(CONFIG_PATH)?;
        let config = match frame_selected {
            Some(color) => self
                .frame_selected_regex()
                .replace(&config, format!("${{1}}{}", color)),
            None => config.into(),
        };
        let config = match frame_highlight {
            Some(color) => self
                .frame_highlight_regex()
                .replace(&config, format!("${{1}}{}", color)),
            None => config,
        };
        std::fs::write(CONFIG_PATH, config.as_bytes())
    }

    fn handle_normal_mode(&mut self, new_selected: u8) {
        let write_result = if new_selected != self.frame_selected {
            self.write_updated_config(Some(new_selected), Some(new_selected))
        } else {
            self.write_updated_config(None, Some(new_selected))
        };
        let _ = write_result;
        self.frame_selected = new_selected;
        self.frame_highlight = new_selected;
    }

    fn handle_locked_mode(&mut self, new_selected: u8) {
        if new_selected != self.frame_selected {
            let _ = self.write_updated_config(Some(new_selected), None);
            self.frame_selected = new_selected;
        }
    }

    fn handle_highlight_mode(&mut self, new_highlight: u8) {
        if new_highlight != self.frame_highlight {
            let _ = self.write_updated_config(None, Some(new_highlight));
            self.frame_highlight = new_highlight;
        }
    }
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        subscribe(&[EventType::ModeUpdate]);
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
            PermissionType::FullHdAccess,
        ]);

        self.re_frame_highlight =
            Some(Regex::new(r"(?ms)(frame_highlight\s*\{.*?\bbase\s+)(\d+)").unwrap());
        self.re_frame_selected =
            Some(Regex::new(r"(?ms)(frame_selected\s*\{.*?\bbase\s+)(\d+)").unwrap());
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
