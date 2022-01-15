use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use soloud::{AudioExt, LoadExt};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};

use std::process::exit;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    Command,
    RawMode,
    Terminal,
}

struct App {
    countdown_seconds: i32,
    countdown_config: i32,
    reset: bool,
    auto_mode: bool,
    sound_played: bool,
    soloud: soloud::Soloud,
    wav: soloud::audio::Wav,
}

impl App {
    fn new(seconds: i32) -> App {
        App {
            countdown_seconds: seconds,
            countdown_config: seconds,
            reset: false,
            auto_mode: false,
            sound_played: false,
            soloud: soloud::Soloud::default().unwrap(),
            wav: soloud::audio::Wav::default(),
        }
    }

    fn on_tick(&mut self) {
        if self.countdown_seconds == 0 && !self.sound_played {
            self.play_sound();
            self.sound_played = true;
        }

        if self.countdown_seconds == 0 && self.auto_mode {
            self.reset = true;
        }

        if self.reset {
            self.countdown_seconds = self.countdown_config;
            self.sound_played = false;
            self.reset = false;
        }

        if self.countdown_seconds > 0 {
            self.countdown_seconds -= 1;
        }
    }

    fn get_hhmmss(&self) -> String {
        let hours = self.countdown_seconds / 3600;
        let minutes = self.countdown_seconds % 3600 / 60;
        let seconds = self.countdown_seconds % 3600 % 60;

        return format!(
            "{:02}:{:02}:{:02} ({})",
            hours, minutes, seconds, self.countdown_seconds
        );
    }

    fn get_automode_string(&self) -> String {
        return format!("Auto mode: {}", self.auto_mode);
    }

    fn reset(&mut self) {
        self.reset = true
    }

    fn toggle_auto_mode(&mut self) {
        self.auto_mode = !self.auto_mode;
    }

    fn play_sound(&mut self) {
        let handle = self.soloud.play(&self.wav);
        self.soloud.set_volume(handle, 0.2f32);

        while self.soloud.voice_count() > 0 {}
    }
}

fn play_sound(sl: &soloud::Soloud) {
    //     let mut sl = soloud::Soloud::default().unwrap();
    //     let mut wav = soloud::audio::Wav::default();

    //     let _ = wav
    //         .load_mem(include_bytes!("../resources/chimes.wav"))
    //         .unwrap();
    // let handle = sl.play(&wav);

    // VOLUME BETWEEN 0.0 AND 1.0!!! DONT BLOW YOUR EARDRUMS!!
    // sl.set_volume(handle, 0.2f32);

    // while sl.voice_count() > 0 {}
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Requires only 1 argument: time in seconds");
        exit(1);
    }

    let time_seconds: i32 = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("Failure parsing argument: it must be a number");
            exit(1);
        }
    };

    enable_raw_mode().map_err(|_| Error::RawMode)?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|_| Error::Command)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|_| Error::Terminal)?;

    // create app and run it
    let mut app = App::new(time_seconds);

    // load wav into memory
    let _ = app
        .wav
        .load_mem(include_bytes!("../resources/chimes.wav"))
        .unwrap();

    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode().map_err(|_| Error::RawMode)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|_| Error::Command)?;

    terminal.show_cursor().map_err(|_| Error::Terminal)?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> std::io::Result<()> {
    let tick_rate = std::time::Duration::from_millis(1000);
    let mut last_tick = std::time::Instant::now();

    loop {
        terminal.draw(|f| ui(f, &app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| std::time::Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }

                if let KeyCode::Char('r') = key.code {
                    app.reset();
                }

                if let KeyCode::Char('a') = key.code {
                    app.toggle_auto_mode();
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = std::time::Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([tui::layout::Constraint::Length(0)].as_ref())
        .split(f.size());

    let title = app.get_hhmmss();
    let automode = app.get_automode_string();
    let text = vec![
        tui::text::Spans::from(title),
        tui::text::Spans::from(automode),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title(tui::text::Span::styled(
            "",
            tui::style::Style::default()
                .fg(tui::style::Color::Magenta)
                .add_modifier(tui::style::Modifier::BOLD),
        ));

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(tui::layout::Alignment::Center)
        .wrap(tui::widgets::Wrap { trim: true });

    f.render_widget(paragraph, chunks[0]);
}
