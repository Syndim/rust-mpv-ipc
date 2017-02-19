extern crate mpv_ipc;

use std::thread;
use std::time::Duration;
use mpv_ipc::{MpvClient, LoadMode};

fn main() {
    let mut mpv = MpvClient::new("/tmp/mpvsocket").unwrap();
    mpv.load_file("/home/syndim/code/7.mp3", LoadMode::Replace).unwrap();
    thread::sleep(Duration::from_millis(10000));
    mpv.pause().unwrap();
    thread::sleep(Duration::from_millis(2000));
    mpv.resume().unwrap();
    let position = mpv.get_position().unwrap().unwrap();
    // let duration = mpv.get_duration().unwrap().unwrap();
    println!("{}", position);
}