// TODO: Media controls metadata
// TODO: Bigger playlist
// TODO: Make song downloads non-blocking

use std::{
    collections::VecDeque,
    fs::File,
    path::PathBuf,
    process,
    str::FromStr,
    sync::mpsc::{self, Receiver},
};

use anyhow::{anyhow, Result};
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use twitcheventsub::{MessageData, TwitchEventSubApi};
use url::Url;

use crate::command::{Command, CommandArgsResult};

fn config_default_playlist_blacklist_previous_songs_len() -> usize {
    5
}

fn config_default_audio_format() -> String {
    "vorbis".to_owned()
}

fn config_default_audio_format_ext() -> String {
    "ogg".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioConfig {
    playlist: String,
    #[serde(rename = "playlist-path")]
    playlist_path: PathBuf,
    #[serde(
        rename = "playlist-blacklist-previous-songs-len",
        default = "config_default_playlist_blacklist_previous_songs_len"
    )]
    playlist_blacklist_previous_songs_len: usize,
    #[serde(rename = "requested-path")]
    requested_path: PathBuf,
    #[serde(rename = "audio-format", default = "config_default_audio_format")]
    audio_format: String,
    #[serde(
        rename = "audio-format-ext",
        default = "config_default_audio_format_ext"
    )]
    audio_format_ext: String,
}

#[allow(dead_code)]
pub struct Radio {
    config: RadioConfig,
    #[allow(unused)]
    stream: rodio::OutputStream,
    #[allow(unused)]
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
    played: Vec<RadioPlatformSong>,
    queue: VecDeque<RadioPlatformSong>,
    controls: souvlaki::MediaControls,
    rx: Receiver<souvlaki::MediaControlEvent>,
}

impl Radio {
    pub fn initialize(config: RadioConfig) -> Result<Self> {
        process::Command::new("yt-dlp")
            .stdout(process::Stdio::inherit())
            .arg("-x")
            .args(["--audio-format", &config.audio_format])
            .args(["-o", "%(extractor)s-%(id)s.%(ext)s"])
            .args(["--paths", config.playlist_path.to_str().unwrap()])
            .args([
                "--download-archive",
                &format!("{}/archive.txt", config.playlist_path.to_str().unwrap()),
            ])
            .arg(&config.playlist)
            .output()?;

        let (stream, stream_handle) = rodio::OutputStream::try_default()?;
        let sink = rodio::Sink::try_new(&stream_handle)?;
        sink.set_volume(0.25);
        let mut controls = souvlaki::MediaControls::new(souvlaki::PlatformConfig {
            display_name: "vulae-twitch-bot",
            dbus_name: "vulae-twitch-bot",
            hwnd: None,
        })?;
        let (tx, rx) = mpsc::channel();
        controls.attach(move |event| {
            tx.send(event).unwrap();
        })?;
        // Needs to have set metadata for events to start being recieved.
        controls.set_metadata(Default::default())?;
        Ok(Self {
            config,
            stream,
            stream_handle,
            sink,
            played: Vec::new(),
            queue: VecDeque::new(),
            controls,
            rx,
        })
    }

    fn load_next_song(&mut self, song_path: &PathBuf) -> Result<()> {
        println!("Load: {:?}", song_path);
        let source = rodio::Decoder::new(File::open(song_path)?)?;
        self.sink.append(source);
        self.sink.play();
        let platform_song =
            RadioPlatformSong::from_filename(song_path.file_name().unwrap().to_str().unwrap())
                .unwrap();
        self.played.push(platform_song.clone());
        if self.played.len() > self.config.playlist_blacklist_previous_songs_len {
            // Sorry
            self.played.remove(0);
        }
        self.queue.push_back(platform_song);
        Ok(())
    }

    fn load_random_next_song(&mut self) -> Result<()> {
        // All playlist songs
        let mut song_paths = std::fs::read_dir(&self.config.playlist_path)?
            .filter_map(|entry| {
                let Ok(entry) = entry else {
                    return None;
                };
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some(&self.config.audio_format_ext)
                {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        // Filter out just played songs
        self.played[self
            .played
            .len()
            .saturating_sub(self.config.playlist_blacklist_previous_songs_len)..]
            .iter()
            .for_each(|previous| {
                song_paths.retain(|song_path| {
                    let Some(platform_song) = RadioPlatformSong::from_filename(
                        song_path.file_name().unwrap().to_str().unwrap(),
                    ) else {
                        println!("This should never happen.");
                        return false;
                    };
                    *previous != platform_song
                });
            });

        // Random song
        let song_path = song_paths.choose(&mut rand::thread_rng()).unwrap();

        self.load_next_song(song_path)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Use new_* for sanitized constructor
pub enum RadioPlatformSong {
    YouTube { id: String },
}

impl RadioPlatformSong {
    fn new_youtube(id: &str) -> Result<Self> {
        const ALLOWED_CHARS: &str =
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
        if id.chars().all(|c| ALLOWED_CHARS.contains(c)) {
            Ok(RadioPlatformSong::YouTube { id: id.to_owned() })
        } else {
            Err(anyhow!("Invalid YouTube ID"))
        }
    }

    fn from_filename(filename: &str) -> Option<RadioPlatformSong> {
        match filename.split("-").next() {
            Some("youtube") => Some(RadioPlatformSong::YouTube {
                id: filename
                    .split("-")
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("-")
                    .split(".")
                    .next()?
                    .to_owned(),
            }),
            _ => None,
        }
    }

    fn to_filename(&self) -> String {
        match self {
            RadioPlatformSong::YouTube { id } => format!("youtube-{}", id),
        }
    }

    fn to_url(&self) -> Url {
        match self {
            RadioPlatformSong::YouTube { id } => {
                Url::from_str(&format!("https://youtube.com/watch?v={}", id)).unwrap()
            }
        }
    }

    fn apply_yt_dlp<'a>(&self, cmd: &'a mut process::Command) -> &'a mut process::Command {
        cmd.arg(self.to_url().to_string())
    }
}

#[derive(Debug)]
pub enum RadioArgs {
    DisplayCurrentSong,
    SkipCurrentSong,
    SongRequest(RadioPlatformSong),
}

impl Command<RadioArgs> for Radio {
    fn parse_args(&self, chat_message: &MessageData) -> CommandArgsResult<RadioArgs> {
        let mut split = chat_message.message.text.split(" ");
        match split.next() {
            Some("!currentsong") | Some("!song") => {
                CommandArgsResult::Execute(RadioArgs::DisplayCurrentSong)
            }
            Some("!skipsong") | Some("!skip") => {
                CommandArgsResult::Execute(RadioArgs::SkipCurrentSong)
            }
            Some("!songrequest") | Some("!sr") => {
                let Some(url_str) = split.next() else {
                    return CommandArgsResult::BadArguments("Must include URL".to_owned());
                };
                let Ok(url) = Url::parse(url_str) else {
                    return CommandArgsResult::BadArguments("Invalid URL".to_owned());
                };
                match url.domain() {
                    Some("www.youtube.com") => {
                        let Some((_, id)) = url.query_pairs().find(|(key, _)| key == "v") else {
                            return CommandArgsResult::BadArguments(
                                "Could not extract YouTube video ID from URL".to_owned(),
                            );
                        };
                        if let Ok(song) = RadioPlatformSong::new_youtube(&id) {
                            CommandArgsResult::Execute(RadioArgs::SongRequest(song))
                        } else {
                            CommandArgsResult::BadArguments("Invalid YouTube ID".to_owned())
                        }
                    }
                    Some("youtu.be") => {
                        let Some(id) = url.path_segments().and_then(|mut segments| segments.next())
                        else {
                            return CommandArgsResult::BadArguments(
                                "Could not extract YouTube video ID from URL".to_owned(),
                            );
                        };
                        if let Ok(song) = RadioPlatformSong::new_youtube(id) {
                            CommandArgsResult::Execute(RadioArgs::SongRequest(song))
                        } else {
                            CommandArgsResult::BadArguments("Invalid YouTube ID".to_owned())
                        }
                    }
                    _ => CommandArgsResult::BadArguments("Unsupported platform".to_owned()),
                }
            }
            _ => CommandArgsResult::WrongCommand,
        }
    }

    fn execute(
        &mut self,
        args: RadioArgs,
        chat_message: &MessageData,
        api: &mut TwitchEventSubApi,
    ) -> Result<()> {
        match args {
            RadioArgs::DisplayCurrentSong => {
                if let Some(current_song) = self.queue.front() {
                    let _ = api.send_chat_message_with_reply(
                        current_song.to_url().to_string(),
                        Some(chat_message.message_id.clone()),
                    );
                }
            }
            RadioArgs::SongRequest(platform_song) => {
                println!(
                    "{} ({}) requested: {}",
                    chat_message.chatter.name,
                    chat_message.chatter.id,
                    platform_song.to_url()
                );

                platform_song
                    .apply_yt_dlp(
                        process::Command::new("yt-dlp")
                            .stdout(process::Stdio::inherit())
                            .arg("-x")
                            .args(["--audio-format", &self.config.audio_format])
                            .args(["-o", "%(extractor)s-%(id)s.%(ext)s"])
                            .args(["--paths", self.config.requested_path.to_str().unwrap()])
                            .args([
                                "--download-archive",
                                &format!(
                                    "{}/archive.txt",
                                    self.config.requested_path.to_str().unwrap()
                                ),
                            ]),
                    )
                    .output()?;

                let mut song_path = self.config.requested_path.clone();
                song_path.push(platform_song.to_filename());
                song_path.set_extension(&self.config.audio_format_ext);
                self.load_next_song(&song_path)?;
            }
            RadioArgs::SkipCurrentSong => {
                self.sink.skip_one();
            }
        }
        Ok(())
    }

    fn update(&mut self, _api: &mut TwitchEventSubApi) -> Result<()> {
        match self.rx.try_recv() {
            Ok(event) => match event {
                souvlaki::MediaControlEvent::Toggle => {
                    if self.sink.is_paused() {
                        self.sink.play();
                    } else {
                        self.sink.pause();
                    }
                }
                souvlaki::MediaControlEvent::Next => {
                    self.sink.skip_one();
                }
                souvlaki::MediaControlEvent::Previous => {
                    println!("Media control previous not implemented.")
                }
                event => println!("Unimplemented event {:#?}", event),
            },
            Err(mpsc::TryRecvError::Empty) => {}
            Err(err) => return Err(err.into()),
        }

        if self.sink.empty() {
            self.queue.pop_front();
            self.load_random_next_song()?;
        }
        Ok(())
    }
}

//#[cfg(test)]
//mod test {
//    use std::time::Duration;
//
//    use anyhow::Result;
//    use twitcheventsub::MessageData;
//
//    use crate::{
//        command::{Command, CommandArgsResult},
//        commands,
//    };
//
//    fn create_test_message(content: &str) -> MessageData {
//        serde_json::from_str(&format!("{{\"broadcaster_user_id\":\"1131985206\",\"broadcaster_user_name\":\"Vulae_\",\"broadcaster_user_login\":\"vulae_\",\"chatter_user_id\":\"1131985206\",\"chatter_user_name\":\"Vulae_\",\"chatter_user_login\":\"vulae_\",\"message_id\":\"52c08310-98e2-42f3-9300-2581b51d5dca\",\"message\":{{\"text\":\"{}\",\"fragments\":[{{\"type\":\"text\",\"text\":\"{}\",\"cheermote\":null,\"emote\":null,\"mention\":null}}]}},\"color\":\"\",\"badges\":[{{\"set_id\":\"broadcaster\",\"id\":\"1\",\"info\":\"\"}}],\"message_type\":\"text\",\"cheer\":null,\"reply\":null,\"channel_points_custom_reward_id\":null,\"channel_points_animation_id\":null}}", content, content)).unwrap()
//    }
//
//    #[test]
//    fn radio_command() -> Result<()> {
//        let mut radio = commands::Radio::initialize()?;
//
//        let chat_message = create_test_message("!sr https://www.youtube.com/watch?v=dQw4w9WgXcQ");
//        if let CommandArgsResult::Execute(args) = radio.parse_args(&chat_message) {
//            radio.execute(args, &chat_message)?;
//        }
//
//        std::thread::sleep(Duration::from_millis(1000));
//        let chat_message = create_test_message("!skip");
//        if let CommandArgsResult::Execute(args) = radio.parse_args(&chat_message) {
//            radio.execute(args, &chat_message)?;
//        }
//
//        std::thread::sleep(Duration::from_millis(100));
//        radio.update()?;
//
//        std::thread::sleep(Duration::from_millis(1000));
//
//        Ok(())
//    }
//}
