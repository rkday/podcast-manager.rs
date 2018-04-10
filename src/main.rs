extern crate rss;
extern crate htmlescape;
extern crate textwrap;
extern crate termion;
extern crate reqwest;
extern crate url;
use rss::Channel;
extern crate tui;

use std::io;
use std::fs::File;
use std::io::prelude::*;

use std::path::PathBuf;
use std::thread;
use std::time;
use std::sync::mpsc;

use url::{Url, ParseError};

use termion::event;
use termion::input::TermRead;

use tui::Terminal;
use tui::backend::RawBackend;
use tui::widgets::{Block, Borders, SelectableList, Widget, Paragraph};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};

enum Event {
    Input(event::Key),
    Tick,
}

struct PodcastData {
    items: Vec<rss::Item>,
    selected: usize,
    size: tui::layout::Rect,
}

fn main() {
    let channel = Channel::from_url("http://www.angryweasel.com/ABTesting/feed/").unwrap();
    let episodes = channel.items().to_vec();
    let episodes_iter = episodes.into_iter();
    let audio_episodes: Vec<rss::Item> = episodes_iter.filter(|i| item_is_audio(i)).collect();

    let mut terminal = init().expect("Failed initialization");
    let size = terminal.size().unwrap();
    let mut pd = PodcastData{items: audio_episodes, selected: 4, size: size};

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

    draw(&mut terminal, &pd);

    loop {
        let size = terminal.size().unwrap();
        if size != pd.size {
            terminal.resize(size).unwrap();
            pd.size = size;
        }

        let evt = rx.recv().unwrap();
        match evt {
            Event::Input(input) => match input {
                event::Key::Char('q') => {
                    break;
                }
                event::Key::Char('y') => {
                    let url = pd.items[pd.selected].enclosure().unwrap().url();
                    let parsed_url = Url::parse(url).unwrap();
                    let url_path = parsed_url.path();
                    let filename = std::path::Path::new(url_path).file_name().unwrap();
                    let mut dl_path = PathBuf::new();
                    dl_path.push("/tmp");
                    dl_path.push(filename);
                    let mut resp = reqwest::get(url).expect("Failed to send request");
                    let mut f = File::create(dl_path).expect("Failed to create file");
                    std::io::copy(&mut resp, &mut f);
                }
                event::Key::Down => {
                    pd.selected += 1;
                    if pd.selected > pd.items.len() - 1 {
                        pd.selected = 0;
                    }
                }
                event::Key::Up => if pd.selected > 0 {
                    pd.selected -= 1;
                } else {
                    pd.selected = pd.items.len() - 1;
                },
                _ => {}
            },
            Event::Tick => {
                //app.advance();
            }
        }
        draw(&mut terminal, &pd);
    }
    terminal.clear().unwrap();
}

fn init() -> Result<Terminal<RawBackend>, io::Error> {
    let backend = RawBackend::new()?;
    Terminal::new(backend)
}


fn draw(t: &mut Terminal<RawBackend>, podcasts: &PodcastData) {
    let ep = &podcasts.items[podcasts.selected];
    let desc = ep.description().unwrap_or("(no description)");
    let desc_decoded = htmlescape::decode_html(desc).unwrap();
    let text = format!("Description:\n{}\n\nURL: {}\n\n", desc_decoded, ep.enclosure().unwrap().url());

    Group::default()
        .direction(Direction::Horizontal)
        .sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, &podcasts.size, |t, chunks| {
            let style = Style::default().fg(Color::Black).bg(Color::White);
            SelectableList::default()
                .block(Block::default().borders(Borders::ALL).title("List"))
                .items(&podcasts.items.iter().map(|ep| ep.title().unwrap_or("no title")).collect::<Vec<_>>())
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

fn item_is_audio(item: &rss::Item) -> bool {
    match item.enclosure() {
        None => false,
        Some(file) => file.mime_type() == "audio/mpeg"
    }
}
