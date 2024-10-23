use std::{collections::VecDeque, fs::File, path::PathBuf, process, str::FromStr};

use anyhow::Result;
use rand::prelude::SliceRandom;
use twitcheventsub::{MessageData, TwitchEventSubApi};
use url::Url;

use crate::command::{Command, CommandArgsResult};

static PLAYLIST: &str = "https://www.youtube.com/playlist?list=PLBXgEHtQmuZmH4Nqcnz1ZVdgcqzrCewl_";
static PLAYLIST_PATH: &str = "/home/vulae/Music/vulae-twitch-bot/playlist";
static REQUESTED_PATH: &str = "/home/vulae/Music/vulae-twitch-bot/requests";
static AUDIO_FORMAT: &str = "vorbis";
static AUDIO_FORMAT_EXT: &str = "ogg";

pub struct Radio {
    #[allow(unused)]
    stream: rodio::OutputStream,
    #[allow(unused)]
    stream_handle: rodio::OutputStreamHandle,
    sink: rodio::Sink,
    queue: VecDeque<RadioPlatformSong>,
}

impl Radio {
    pub fn initialize() -> Result<Self> {
        process::Command::new("yt-dlp")
            .stdout(process::Stdio::inherit())
            .arg("-x")
            .args(["--audio-format", AUDIO_FORMAT])
            .args(["-o", "%(extractor)s-%(id)s.%(ext)s"])
            .args(["--paths", PLAYLIST_PATH])
            .args([
                "--download-archive",
                &format!("{}/archive.txt", PLAYLIST_PATH),
            ])
            .arg(PLAYLIST)
            .output()?;

        let (stream, stream_handle) = rodio::OutputStream::try_default()?;
        let sink = rodio::Sink::try_new(&stream_handle)?;
        sink.set_volume(0.25);
        Ok(Self {
            stream,
            stream_handle,
            sink,
            queue: VecDeque::new(),
        })
    }

    fn load_next_song(&mut self, song_path: &PathBuf) -> Result<()> {
        println!("Load: {:?}", song_path);
        let source = rodio::Decoder::new(File::open(song_path)?)?;
        self.sink.append(source);
        self.sink.play();
        self.queue.push_back(
            RadioPlatformSong::from_filename(song_path.file_name().unwrap().to_str().unwrap())
                .unwrap(),
        );
        Ok(())
    }

    fn load_random_next_song(&mut self) -> Result<()> {
        let song_paths = std::fs::read_dir(PLAYLIST_PATH)?
            .filter_map(|entry| {
                let Ok(entry) = entry else {
                    return None;
                };
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("ogg") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let song_path = song_paths.choose(&mut rand::thread_rng()).unwrap();
        self.load_next_song(song_path)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum RadioPlatformSong {
    YouTube { id: String },
}

impl RadioPlatformSong {
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
                        CommandArgsResult::Execute(RadioArgs::SongRequest(
                            RadioPlatformSong::YouTube { id: id.to_string() },
                        ))
                    }
                    Some("youtu.be") => {
                        let Some(id) = url.path_segments().and_then(|mut segments| segments.next())
                        else {
                            return CommandArgsResult::BadArguments(
                                "Could not extract YouTube video ID from URL".to_owned(),
                            );
                        };
                        CommandArgsResult::Execute(RadioArgs::SongRequest(
                            RadioPlatformSong::YouTube { id: id.to_owned() },
                        ))
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
                platform_song
                    .apply_yt_dlp(
                        process::Command::new("yt-dlp")
                            .stdout(process::Stdio::inherit())
                            .arg("-x")
                            .args(["--audio-format", AUDIO_FORMAT])
                            .args(["-o", "%(extractor)s-%(id)s.%(ext)s"])
                            .args(["--paths", REQUESTED_PATH])
                            .args([
                                "--download-archive",
                                &format!("{}/archive.txt", REQUESTED_PATH),
                            ]),
                    )
                    .output()?;

                let mut song_path = PathBuf::from(REQUESTED_PATH);
                song_path.push(platform_song.to_filename());
                song_path.set_extension(AUDIO_FORMAT_EXT);
                self.load_next_song(&song_path)?;
            }
            RadioArgs::SkipCurrentSong => {
                self.sink.skip_one();
            }
        }
        Ok(())
    }

    fn update(&mut self, _api: &mut TwitchEventSubApi) -> Result<()> {
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
