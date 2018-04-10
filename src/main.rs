extern crate rss;
extern crate textwrap;
extern crate termion;
extern crate tui;
extern crate htmlescape;
extern crate reqwest;
extern crate url;

mod podcast_data;

use std::io;

use std::thread;
use std::time;
use std::sync::mpsc;

use termion::event;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::RawBackend;
use tui::widgets::{Block, Borders, SelectableList, Widget, Paragraph};
use tui::layout::{Direction, Group, Size};
use tui::style::{Color, Modifier, Style};

enum Event {
    Input(event::Key),
    Tick,
}

struct Application {
    podcasts: Vec<podcast_data::PodcastData>,
    selected: usize,
    size: tui::layout::Rect,
}

impl Application {
    fn selected_podcast(&self) -> &podcast_data::PodcastData { &self.podcasts[self.selected] }
    fn selected_podcast_mut(&mut self) -> &mut podcast_data::PodcastData { &mut self.podcasts[self.selected] }
    fn selected_episode(&self) -> &podcast_data::PodcastEpisode { self.selected_podcast().selected_episode() }
    fn next_episode(&mut self) -> () { self.selected_podcast_mut().next_episode() }
    fn prev_episode(&mut self) -> () { self.selected_podcast_mut().prev_episode() }
}

fn main() {
    let channel = rss::Channel::from_url("http://www.angryweasel.com/ABTesting/feed/").unwrap();

    let mut terminal = init().expect("Failed initialization");
    let size = terminal.size().unwrap();
    let mut app = Application{podcasts: vec![podcast_data::PodcastData::new(&channel)], selected: 0, size: size};

    // Channels
    let (tx, rx) = mpsc::channel();
    let input_tx = tx.clone();
    let clock_tx = tx.clone();

    // Input
    thread::spawn(move || {
        let stdin = io::stdin();
        for c in stdin.keys() {
            let evt = c.unwrap();
            input_tx.send(Event::Input(evt)).unwrap();
            if evt == event::Key::Char('q') {
                break;
            }
        }
    });

    // Tick
    thread::spawn(move || loop {
        clock_tx.send(Event::Tick).unwrap();
        thread::sleep(time::Duration::from_millis(500));
    });

    draw(&mut terminal, &app);

    loop {
        let size = terminal.size().unwrap();
        if size != app.size {
            terminal.resize(size).unwrap();
            app.size = size;
        }

        let evt = rx.recv().unwrap();
        match evt {
            Event::Input(input) => match input {
                event::Key::Char('q') => {
                    break;
                }
                event::Key::Char('y') => {
                    app.selected_episode().download();
                }
                event::Key::Down => app.next_episode(),
                event::Key::Up => app.prev_episode(),
                _ => {}
            },
            Event::Tick => {
                //app.advance();
            }
        }
        draw(&mut terminal, &app);
    }
    terminal.clear().unwrap();
}

fn init() -> Result<Terminal<RawBackend>, io::Error> {
    let backend = RawBackend::new()?;
    Terminal::new(backend)
}

fn draw(t: &mut Terminal<RawBackend>, app: &Application) {
    let podcasts = app.selected_podcast();
    let ep = podcasts.selected_episode();
    let text = format!("Description:\n{}\n\nURL: {}\n\n", ep.description, ep.url);

    Group::default()
        .direction(Direction::Horizontal)
        .sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, &app.size, |t, chunks| {
            let style = Style::default().fg(Color::Black).bg(Color::White);
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("List"))
                .items(&podcasts.items.iter().map(|ep| ep.title.clone()).collect::<Vec<_>>())
                .select(podcasts.selected)
                .style(style)
                .highlight_style(style.clone().fg(Color::LightGreen).modifier(Modifier::Bold))
                .highlight_symbol(">")
                .render(t, &chunks[0]);
            Group::default()
                .direction(Direction::Vertical)
                .sizes(&[Size::Percent(50), Size::Percent(50)])
                .render(t, &chunks[1], |t, chunks| {
                    let text2 = format!("{}", textwrap::fill(&text, (chunks[0].width - 3) as usize));
                    Paragraph::default()
                        .block(Block::default().title("Paragraph").borders(Borders::ALL))
                        .style(Style::default().fg(Color::White).bg(Color::Black))
                        .wrap(true)
                        .text(&text2)
                        .render(t, &chunks[0]);
                })

        });

    t.draw().unwrap();
}
