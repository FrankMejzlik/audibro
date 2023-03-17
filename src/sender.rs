//!
//! The main module providing high-level API for the sender of the data.
//!

use std::io::{stdin, Read, Write};
use std::io::{stdout, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver as MpscReceiver, Sender as MpscSender};
use std::sync::Arc;
use std::time::Duration;
use std::vec;
// ---
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::{Attribute, Color, Stylize};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{cursor, execute, queue, style, Result as CrosstermResult};
// ---
#[allow(unused_imports)]
use hashsig::{debug, error, info, log_input, trace, warn};
use hashsig::{Sender, SenderParams, SenderTrait};
// ---
use crate::config::{self, BlockSignerInst};

#[derive(Debug)]
pub struct AudiBroSenderParams {
    pub running: Arc<AtomicBool>,
    pub seed: u64,
    pub layers: usize,
    /// An address where the sender will listen for heartbeats.
    pub addr: String,
    /// A number of signatures one keypair can generate.
    pub key_lifetime: usize,
    pub cert_interval: usize,
    pub max_piece_size: usize,
    pub tui: bool,
}

pub struct AudiBroSender {
    params: AudiBroSenderParams,
    sender: Sender<BlockSignerInst>,
}

impl AudiBroSender {
    pub fn new(params: AudiBroSenderParams) -> Self {
        let sender = Sender::new(SenderParams {
            addr: params.addr.clone(),
            running: params.running.clone(),
            layers: params.layers,
            seed: params.seed,
            id_dir: config::ID_DIR.into(),
            id_filename: config::ID_FILENAME.into(),
            datagram_size: config::DATAGRAM_SIZE,
            net_buffer_size: config::BUFFER_SIZE,
            subscriber_lifetime: config::SUBSCRIBER_LIFETIME,
            key_lifetime: params.key_lifetime,
            cert_interval: params.cert_interval,
            max_piece_size: params.max_piece_size,
        });
        AudiBroSender { params, sender }
    }

    pub fn run(&mut self, input: &mut dyn Read) {
        let (tx, mut rx) = channel();

        std::thread::spawn(move || Self::run_tui(tx));

        // The main loop as long as the app should run
        while self.params.running.load(Ordering::Acquire) {
            // Get the data to broadcast from TUI mode
            let data = if self.params.tui {
                Self::read_input_tui(input, &mut rx)
            }
            // Else get data from stream mode
            else {
                Self::read_input(input)
            };

            if let Err(e) = self.sender.broadcast(data) {
                warn!("Failed to broadcast! ERROR: {e}");
            }
        }
    }
    // ---

    /// Reads the available chunk of data from the provided input.
    fn read_input(_input: &mut dyn Read) -> Vec<u8> {
        let input_bytes;
        #[cfg(feature = "simulate_stdin")]
        {
            use chrono::Local;
            use std::thread;

            if let Some(x) = config::SIM_INPUT_PERIOD {
                // We simulate periodic data coming via input
                thread::sleep(x);
            } else {
                let mut handle = stdin().lock();
                let mut input = String::new();
                handle.read_line(&mut input).expect("Failed to read line");
            }

            let msg = Local::now().format("%d-%m-%Y %H:%M:%S").to_string();
            input_bytes = msg.into_bytes();
        }

        #[cfg(not(feature = "simulate_stdin"))]
        {
            let mut buf = vec![];
            _input.read_to_end(&mut buf).expect("Fail!");
            input_bytes = buf;
        }

        debug!(tag: "broadcasted", "{}", String::from_utf8_lossy(&input_bytes));
        input_bytes
    }

    ///
    /// Runs the TUI and periodically sends the input data to broadcast.
    ///
    fn read_input_tui(_input: &mut dyn Read, rx: &mut MpscReceiver<Vec<u8>>) -> Vec<u8> {
        // Wait for the data
        let input_bytes = match rx.recv() {
            Ok(x) => x,
            Err(_e) => panic!("The input is dead!"),
        };

        debug!(tag: "broadcasted", "{}", String::from_utf8_lossy(&input_bytes));
        input_bytes
    }

    fn run_tui(mut tx: MpscSender<Vec<u8>>) {
        let menu_items = vec![
            vec![
                "Disturbed - Inside the fire",
                "Slipknot - Snuff",
                "Bullet For My Valentine - Hand of Blood",
            ],
            vec!["MICROPHONE"],
            vec!["QUIT"],
        ];
        let menu_items_flat = menu_items
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

            if let Some(x) = read_action() {
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
                        Self::process_menu_item(&mut tx, menu_items_flat[selected_item]);
                        active_item = Some(selected_item);
                    }
                    KeyCode::Char('q') => break,
                    _ => {}
                };
                changed = true;
            }

            info!("selected_item: {selected_item}");
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

    fn process_menu_item(tx: &mut MpscSender<Vec<u8>>, item: &str) {
        info!("Processing menu item: {}", item);

        if item == "QUIT" {
            std::process::exit(0x01);
        }

        let x = item.to_ascii_lowercase().into_bytes();

        match tx.send(x) {
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
