extern crate rss;
extern crate htmlescape;
use rss::Channel;

fn item_is_audio(item: &rss::Item) -> bool {
    match item.enclosure() {
        None => false,
        Some(file) => file.mime_type() == "audio/mpeg"
    }
}

fn main() {
    println!("Hello, world!");
    let channel = Channel::from_url("http://www.angryweasel.com/ABTesting/feed/").unwrap();
    let episodes = channel.items().to_vec();
    let episodes_iter = episodes.into_iter();
    let audio_episodes: Vec<rss::Item> = episodes_iter.filter(|i| item_is_audio(i)).collect();
    for ep in audio_episodes {
        let desc = ep.description().unwrap_or("(no description)");
        let desc_decoded = htmlescape::decode_html(desc).unwrap();
        println!("{}\n{}\n\n", desc_decoded, ep.enclosure().unwrap().url());
    }
}
