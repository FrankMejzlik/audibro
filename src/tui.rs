use std::io::stdout;
use std::io::Write;
use std::sync::mpsc::{channel, Receiver as MpscReceiver, Sender as MpscSender};
use std::time::Duration;
use std::vec;
// ---
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::{Attribute, Color, Stylize};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{cursor, execute, queue, style};

#[allow(unused_imports)]
use hab::{debug, error, info, log_input, trace, warn};
// ---
use crate::audio_source::{AudioFile, AudioSource, AudioSourceData};
use crate::config;

pub struct TerminalUiReceiver {
    state_rx: MpscReceiver<String>,
    addr: String,
    name: String,
    distribute: bool,
}

impl TerminalUiReceiver {
    pub fn new(
        state_rx: MpscReceiver<String>,
        addr: String,
        name: String,
        distribute: bool,
    ) -> Self {
        Self {
            state_rx,
            addr,
            name,
            distribute,
        }
    }

    pub fn run_tui(&self) {
        let menu_items = vec![vec!["QUIT".into()]];
        let menu_items_data = vec![vec!["QUIT".into()]];
        let menu_items_flat = menu_items
            .clone()
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();
        let menu_items_data_flat = menu_items_data
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();

        let mut selected_item: usize = 0;
        let mut active_item = None;

        let mut changed = true;
        let mut auth_state = config::WAITING_FOR_DATA.to_string();

        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All)).unwrap();
        loop {
            if let Ok(state) = self.state_rx.try_recv() {
                auth_state = state;
                changed = true;
            }
            if changed {
                let distr_string = if self.distribute {
                    "    >>> DISTRIBUTING DATA <<<"
                } else {
                    "    --- NOT DISTRIBUTING DATA ---"
                };
                let state_string = match auth_state.as_str() {
                    "Authenticated" => format!("       {}       ", auth_state)
                        .white()
                        .on(Color::Green),
                    "Certified" => format!("       {}       ", auth_state)
                        .white()
                        .on(Color::Yellow),
                    "Unverified" => format!("       {}       ", auth_state)
                        .white()
                        .on(Color::Red),
                    _ => format!("       {}       ", auth_state).white(),
                };
                queue!(
                    stdout,
                    style::ResetColor,
                    terminal::Clear(ClearType::All),
                    cursor::Hide,
                    cursor::MoveTo(0, 1),
                    style::Print("++++++++++++++++++++++++++++++++++++++++++"),
                    cursor::MoveToNextLine(1),
                    style::Print(format!("    Listening to {} @ {}", self.name, self.addr)),
                    cursor::MoveToNextLine(1),
                    style::Print("++++++++++++++++++++++++++++++++++++++++++"),
                    cursor::MoveToNextLine(2),
                    style::Print(distr_string),
                    cursor::MoveToNextLine(1),
                    style::Print("------------------------------------"),
                    cursor::MoveToNextLine(1),
                    style::PrintStyledContent(state_string),
                    cursor::MoveToNextLine(1),
                    style::Print("------------------------------------"),
                    cursor::MoveToNextLine(2),
                )
                .unwrap();

                let mut i = 0;
                for items in menu_items.iter() {
                    for item in items.iter() {
                        let cursor = if i == selected_item { ">>" } else { "  " };
                        let active = if let Some(x) = active_item {
                            if i == x {
                                let cursor = if i == selected_item { ">>" } else { "->" };
                                style::PrintStyledContent(
                                    format!("{} {} ", cursor, item)
                                        .with(Color::Cyan)
                                        .attribute(Attribute::Bold),
                                )
                            } else {
                                style::PrintStyledContent(format!("{} {}", cursor, item).white())
                            }
                        } else {
                            style::PrintStyledContent(format!("{} {}", cursor, item).white())
                        };
                        queue!(stdout, active, cursor::MoveToNextLine(1)).unwrap();
                        i += 1;
                    }
                    queue!(stdout, style::Print("---"), cursor::MoveToNextLine(1)).unwrap();
                }
                stdout.flush().unwrap();
                changed = false;
            }

            if let Some(x) = read_action() {
                match x {
                    KeyCode::Up => {
                        selected_item = selected_item.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if selected_item < menu_items_flat.len() - 1 {
                            selected_item += 1;
                        }
                    }
                    KeyCode::Enter => {
                        self.process_menu_item(&menu_items_data_flat[selected_item]);
                        active_item = Some(selected_item);
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                };
                changed = true;
            }
        }
        execute!(
            stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .unwrap();
        disable_raw_mode().unwrap();
        self.process_menu_item("QUIT");
    }

    fn process_menu_item(&self, item: &str) {
        info!("Processing menu item: {}", item);

        if item == "QUIT" {
            std::process::exit(0x01);
        }
    }
}

pub struct TerminalUi {
    _audio_src: AudioSource,
    audio_src_tx: MpscSender<AudioSourceData>,
}

impl TerminalUi {
    pub fn new(data_tx: MpscSender<Vec<u8>>) -> Self {
        let (tx, rx) = channel();
        Self {
            _audio_src: AudioSource::new(rx, data_tx),
            audio_src_tx: tx,
        }
    }
    pub fn run_tui(&self, audio_options: &[AudioFile]) {
        let mut audio_menu = vec![];
        let mut audio_files = vec![];

        for audio_file in audio_options.iter() {
            audio_menu.push(format!("{} - {}", audio_file.artist, audio_file.title));
            audio_files.push(audio_file.filepath.clone());
        }

        let menu_items = vec![audio_menu, vec!["MICROPHONE".into()], vec!["QUIT".into()]];
        let menu_items_data = vec![audio_files, vec!["MICROPHONE".into()], vec!["QUIT".into()]];
        let menu_items_flat = menu_items
            .clone()
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();
        let menu_items_data_flat = menu_items_data
            .into_iter()
            .flatten()
            .collect::<Vec<String>>();

        let mut selected_item: usize = 0;
        let mut active_item = None;

        let mut changed = true;

        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All)).unwrap();
        loop {
            if changed {
                queue!(
                    stdout,
                    style::ResetColor,
                    terminal::Clear(ClearType::All),
                    cursor::Hide,
                    cursor::MoveTo(1, 1),
                    style::Print("Choose broadcast input:\n\r-------------------------"),
                    cursor::MoveToNextLine(1)
                )
                .unwrap();

                let mut i = 0;
                for items in menu_items.iter() {
                    for item in items.iter() {
                        let cursor = if i == selected_item { ">>" } else { "  " };
                        let active = if let Some(x) = active_item {
                            if i == x {
                                let cursor = if i == selected_item { ">>" } else { "->" };
                                style::PrintStyledContent(
                                    format!("{} {} ", cursor, item)
                                        .with(Color::Cyan)
                                        .attribute(Attribute::Bold),
                                )
                            } else {
                                style::PrintStyledContent(format!("{} {}", cursor, item).white())
                            }
                        } else {
                            style::PrintStyledContent(format!("{} {}", cursor, item).white())
                        };
                        queue!(stdout, active, cursor::MoveToNextLine(1)).unwrap();
                        i += 1;
                    }
                    queue!(stdout, style::Print("---"), cursor::MoveToNextLine(1)).unwrap();
                }
                stdout.flush().unwrap();
                changed = false;
            }

            if let Some(x) = read_action() {
                match x {
                    KeyCode::Up => {
                        selected_item = selected_item.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if selected_item < menu_items_flat.len() - 1 {
                            selected_item += 1;
                        }
                    }
                    KeyCode::Enter => {
                        self.process_menu_item(&menu_items_data_flat[selected_item]);
                        active_item = Some(selected_item);
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                };
                changed = true;
            }
        }
        execute!(
            stdout,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .unwrap();
        disable_raw_mode().unwrap();
        self.process_menu_item("QUIT");
    }

    fn process_menu_item(&self, item: &str) {
        info!("Processing menu item: {}", item);

        if item == "QUIT" {
            std::process::exit(0x01);
        }

        match self.audio_src_tx.send(AudioSourceData::new_file(item)) {
            Ok(x) => x,
            Err(e) => info!("ERROR: {e}"),
        };
    }
}

pub fn read_action() -> Option<KeyCode> {
    if event::poll(Duration::from_millis(500)).unwrap() {
        if let Ok(Event::Key(ev)) = event::read() {
            if let KeyEventKind::Press = ev.kind {
                return Some(ev.code);
            }
        }
    }
    None
}
