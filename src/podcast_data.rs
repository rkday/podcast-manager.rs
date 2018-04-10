extern crate rss;
extern crate htmlescape;
extern crate reqwest;
extern crate url;

use std::path::{Path, PathBuf};
use std::io;
use std::io::copy;
use std::fs::File;
use std::io::prelude::*;
use std::ffi::OsString;
use url::{Url, ParseError};

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
struct PodcastEpisode {
    url: String,
    filename: OsString,
    description: String,
}

impl PodcastEpisode {
    fn new(source: &rss::Item) -> PodcastEpisode {
        let desc = source.description().unwrap_or("(no description)");
        let desc_decoded = htmlescape::decode_html(desc).unwrap();

        let url = source.enclosure().unwrap().url();
        let parsed_url = Url::parse(url).unwrap();
        let url_path = parsed_url.path();
        let filename = Path::new(url_path).file_name().unwrap();
        let mut dl_path = PathBuf::new();

        PodcastEpisode{url: url.to_string(), filename: filename.to_os_string(), description: desc_decoded}
    }

    fn download(&self) -> () {
        let mut dl_path = PathBuf::new();
        dl_path.push("/tmp");
        dl_path.push(&self.filename);
        let mut resp = reqwest::get(&self.url).expect("Failed to send request");
        let mut f = File::create(dl_path).expect("Failed to create file");
        copy(&mut resp, &mut f);
    }
}

struct PodcastData {
    items: Vec<PodcastEpisode>,
    selected: usize,
}

impl PodcastData {
    fn new(channel: &rss::Channel) -> PodcastData {
        let episodes = channel.items().to_vec();
        let episodes_iter = episodes.into_iter();
        let audio_episodes: Vec<rss::Item> = episodes_iter.filter(|i| item_is_audio(i)).collect();
        let processed_episodes: Vec<PodcastEpisode> = audio_episodes.iter().map(|i| PodcastEpisode::new(i)).collect();
        PodcastData{items: processed_episodes, selected: 0}
    }

    fn selected_episode(&self) -> PodcastEpisode { self.items[self.selected].clone() }
}

pub fn item_is_audio(item: &rss::Item) -> bool {
    match item.enclosure() {
        None => false,
        Some(file) => file.mime_type() == "audio/mpeg"
    }
}
