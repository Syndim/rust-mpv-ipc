use std::fmt::{Display, Formatter, Error as FmtError};
use std::string::ToString;
use std::result::Result;

use rand::{self, Rng};
use rustc_serialize::Decodable;
use rustc_serialize::json::Json;

pub enum SeekMode {
    Relative,
    Absolute,
    AbsolutePercent,
    RelativePercent,
    Exact,
    KeyFrames,
}

impl Display for SeekMode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let name = match self {
            &SeekMode::Relative => "relative",
            &SeekMode::Absolute => "absolute",
            &SeekMode::AbsolutePercent => "absolute-percent",
            &SeekMode::RelativePercent => "relative-percent",
            &SeekMode::Exact => "exact",
            &SeekMode::KeyFrames => "keyframes",
        };

        write!(f, "{}", name)
    }
}

pub enum LoadMode {
    Replace,
    Append,
    AppendPlay,
}

impl Display for LoadMode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        let name = match self {
            &LoadMode::Replace => "replace",
            &LoadMode::Append => "append",
            &LoadMode::AppendPlay => "append-play",
        };

        write!(f, "{}", name)
    }
}

pub enum Command {
    Seek(i64, SeekMode),
    LoadFile(String, LoadMode),
    LoadList(String, LoadMode),
    PlaylistClear,
    Quit(i64),
    Stop,
    SetProperty(String, Json),
    GetProperty(String),
}

impl Command {
    pub fn to_command_list(self) -> Vec<Json> {
        match self {
            Command::Seek(seconds, mode) => {
                vec![Json::String(String::from("seek")),
                     Json::I64(seconds),
                     Json::String(mode.to_string())]
            }
            Command::LoadFile(path, mode) => {
                vec![Json::String(String::from("loadfile")),
                     Json::String(path),
                     Json::String(mode.to_string())]
            }
            Command::LoadList(path, mode) => {
                vec![Json::String(String::from("loadlist")),
                     Json::String(path),
                     Json::String(mode.to_string())]
            }
            Command::PlaylistClear => vec![Json::String(String::from("playlist-clear"))],
            Command::Quit(code) => vec![Json::String(String::from("quit")), Json::I64(code)],
            Command::Stop => vec![Json::String(String::from("stop"))],
            Command::SetProperty(name, value) => {
                vec![Json::String(String::from("set_property")), Json::String(name), value]
            }
            Command::GetProperty(name) => {
                vec![Json::String(String::from("get_property")), Json::String(name)]
            }
        }
    }
}

#[derive(RustcEncodable)]
pub struct Request {
    pub command: Vec<Json>,
    pub request_id: u32,
}

impl Request {
    pub fn new(cmd: Command) -> Self {
        Request {
            command: cmd.to_command_list(),
            request_id: rand::thread_rng().gen::<u32>(),
        }
    }
}

#[derive(RustcDecodable)]
pub struct Response<T: Decodable> {
    pub data: Option<T>,
    error: String,
}

impl<T: Decodable> Response<T> {
    pub fn success(&self) -> bool {
        self.error == "success"
    }
}