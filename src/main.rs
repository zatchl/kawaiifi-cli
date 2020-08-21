mod app;
mod widgets;

use app::App;
use crossterm::{
    event::{read, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use kawaiifi::{self, Bss};
use std::{
    collections::HashSet,
    io::{self, Write},
    iter::Iterator,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread::{self, JoinHandle},
    time::Duration,
};
use tui::{backend::CrosstermBackend, Terminal};
use widgets::{BssTableColumnHeader, TableSortOrder};

fn start_scan_thread() -> (Receiver<HashSet<Bss>>, JoinHandle<()>) {
    let (scan_result_tx, scan_result_rx) = mpsc::channel();
    (
        scan_result_rx,
        thread::spawn(move || {
            let interfaces = kawaiifi::interfaces();
            let first_interface = interfaces
                .iter()
                .next()
                .expect("Could not find any Wi-Fi interfaces");

            if let Ok(cached_scan_results) = first_interface.cached_scan_results() {
                scan_result_tx.send(cached_scan_results).unwrap();
            }

            loop {
                if let Ok(scan_results) = first_interface.scan() {
                    scan_result_tx.send(scan_results).unwrap();
                }
                thread::sleep(Duration::from_secs(1));
            }
        }),
    )
}

fn start_input_event_thread() -> (Receiver<Event>, JoinHandle<()>) {
    let (input_event_tx, input_event_rx) = mpsc::channel();
    (
        input_event_rx,
        thread::spawn(move || loop {
            if let Ok(event) = read() {
                if let Err(_) = input_event_tx.send(event) {
                    break;
                }
            } else {
                break;
            }
        }),
    )
}

fn main() -> Result<(), io::Error> {
    let mut app = App::new();

    let (scan_rx, scan_thread) = start_scan_thread();

    let (input_event_rx, input_event_thread) = start_input_event_thread();

    execute!(io::stdout(), EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        // First check for a new scan
        match scan_rx.try_recv() {
            Ok(scan_results) => {
                app.update_scan_results(scan_results.into_iter().collect::<Vec<Bss>>());
                app.render(&mut terminal)?
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        // Then check for any input events
        match input_event_rx.try_recv() {
            Ok(event) => match event {
                Event::Key(key_event) => match key_event.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('b') => {
                        app.sort_bss_table(BssTableColumnHeader::Bssid, TableSortOrder::Descending);
                        app.render(&mut terminal)?;
                    }
                    KeyCode::Char('s') => {
                        app.sort_bss_table(BssTableColumnHeader::Ssid, TableSortOrder::Descending);
                        app.render(&mut terminal)?;
                    }
                    KeyCode::Up => {
                        app.select_previous();
                        app.render(&mut terminal)?;
                    }
                    KeyCode::Down => {
                        app.select_next();
                        app.render(&mut terminal)?;
                    }
                    KeyCode::Enter => {
                        app.focus_next();
                        app.render(&mut terminal)?;
                    }
                    _ => (),
                },
                _ => (),
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(100));
    }

    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();

    Ok(())
}
