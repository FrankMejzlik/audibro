use std::io::stdout;
use std::io::Write;
use std::sync::mpsc::{channel, Sender as MpscSender};
use std::time::Duration;
use std::vec;
// ---
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::{Attribute, Color, Stylize};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{cursor, execute, queue, style};

#[allow(unused_imports)]
use hashsig::{debug, error, info, log_input, trace, warn};
// ---
use crate::audio_source::AudioSource;
use crate::audio_source::AudioSourceData;

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
    pub fn run_tui(&self) {
        let menu_items = vec![
            vec![
                "Disturbed - Inside the fire",
                "Slipknot - Snuff",
                "Bullet For My Valentine - Hand of Blood",
            ],
            vec!["MICROPHONE"],
            vec!["QUIT"],
        ];
        let menu_items_data = vec![
            vec![
                "data/disturbed_inside-the-fire.mp3",
                "data/slipknot_snuff.mp3",
                "data/bullet-for-my-valentine_hand-of-blood.mp3",
            ],
            vec![""],
            vec!["QUIT"],
        ];
        let menu_items_flat = menu_items
            .clone()
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>();
        let menu_items_data_flat = menu_items_data
            .clone()
            .into_iter()
            .flatten()
            .collect::<Vec<&str>>();

        let mut selected_item = 0;
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
                    style::Print("Choose broadcast input:\n-------------------------"),
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

            if let Some(x) = Self::read_action() {
                match x {
                    KeyCode::Up => {
                        if selected_item > 0 {
                            selected_item -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_item < menu_items_flat.len() - 1 {
                            selected_item += 1;
                        }
                    }
                    KeyCode::Enter => {
                        self.process_menu_item(menu_items_data_flat[selected_item]);
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
}
