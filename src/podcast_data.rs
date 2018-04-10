extern crate rss;
extern crate reqwest;
extern crate url;
extern crate htmlescape;

use std::path::{Path, PathBuf};
use std::io::copy;
use std::fs::File;
use std::ffi::OsString;
use url::Url;

#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct PodcastEpisode {
    pub title: String,
    pub url: String,
    pub filename: OsString,
    pub description: String,
}

impl PodcastEpisode {
    pub fn new(source: &rss::Item) -> PodcastEpisode {
        let desc = source.description().unwrap_or("(no description)");
        let desc_decoded = htmlescape::decode_html(desc).unwrap();

        let url = source.enclosure().unwrap().url();
        let parsed_url = Url::parse(url).unwrap();
        let url_path = parsed_url.path();
        let filename = Path::new(url_path).file_name().unwrap();

        PodcastEpisode{title: source.title().unwrap_or("(no title)").to_string(),
                       url: url.to_string(),
                       filename: filename.to_os_string(),
                       description: desc_decoded}
    }

    pub fn download(&self) -> () {
        let mut dl_path = PathBuf::new();
        dl_path.push("/tmp");
        dl_path.push(&self.filename);
        let mut resp = reqwest::get(&self.url).expect("Failed to send request");
        let mut f = File::create(dl_path).expect("Failed to create file");
        copy(&mut resp, &mut f);
    }
}

pub struct PodcastData {
    pub items: Vec<PodcastEpisode>,
    pub selected: usize,
}

impl PodcastData {
    pub fn new(channel: &rss::Channel) -> PodcastData {
        let episodes = channel.items().to_vec();
        let episodes_iter = episodes.into_iter();
        let audio_episodes: Vec<rss::Item> = episodes_iter.filter(|i| item_is_audio(i)).collect();
        let processed_episodes: Vec<PodcastEpisode> = audio_episodes.iter().map(|i| PodcastEpisode::new(i)).collect();
        PodcastData{items: processed_episodes, selected: 0}
    }

    pub fn selected_episode(&self) -> &PodcastEpisode { &self.items[self.selected] }

    pub fn next_episode(&mut self) -> () {
        self.selected += 1;
        if self.selected > self.items.len() - 1 {
            self.selected = 0;
        }
    }

    pub fn prev_episode(&mut self) -> () {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.items.len() - 1;
        }
    }
}

pub fn item_is_audio(item: &rss::Item) -> bool {
    match item.enclosure() {
        None => false,
        Some(file) => file.mime_type() == "audio/mpeg"
    }
}
