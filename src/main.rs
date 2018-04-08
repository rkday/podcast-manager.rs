extern crate rss;
extern crate htmlescape;
extern crate textwrap;
use rss::Channel;
extern crate tui;

use std::io;

use tui::Terminal;
use tui::backend::RawBackend;
use tui::widgets::{Block, Borders, SelectableList, Widget, Paragraph};
use tui::layout::{Direction, Group, Rect, Size};
use tui::style::{Color, Modifier, Style};


struct PodcastData {
    items: Vec<rss::Item>,
    selected: usize,
}

fn main() {
    let channel = Channel::from_url("http://www.angryweasel.com/ABTesting/feed/").unwrap();
    let episodes = channel.items().to_vec();
    let episodes_iter = episodes.into_iter();
    let audio_episodes: Vec<rss::Item> = episodes_iter.filter(|i| item_is_audio(i)).collect();
    let pd = PodcastData{items: audio_episodes, selected: 4};

    let mut terminal = init().expect("Failed initialization");
    let size = terminal.size().unwrap();
    draw(&mut terminal, size, &pd);
    let ten_secs = std::time::Duration::from_millis(10000);
    std::thread::sleep(ten_secs);
    terminal.clear().unwrap();
}

fn init() -> Result<Terminal<RawBackend>, io::Error> {
    let backend = RawBackend::new()?;
    Terminal::new(backend)
}


fn draw(t: &mut Terminal<RawBackend>, size: Rect, podcasts: &PodcastData) {
    let ep = &podcasts.items[podcasts.selected];
    let desc = ep.description().unwrap_or("(no description)");
    let desc_decoded = htmlescape::decode_html(desc).unwrap();
    let text = format!("Description:\n{}\n\nURL: {}\n\n", desc_decoded, ep.enclosure().unwrap().url());

    Group::default()
        .direction(Direction::Horizontal)
        .sizes(&[Size::Percent(50), Size::Percent(50)])
        .render(t, &size, |t, chunks| {
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
                    let text2 = format!("{}", textwrap::fill(&text, chunks[0].width as usize));
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
