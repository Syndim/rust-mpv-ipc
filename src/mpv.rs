use std::io::{Read, Write, Result, Error as IoError, ErrorKind};
use std::os::unix::net::UnixStream;
use rustc_serialize::Decodable;
use rustc_serialize::json::{self, Json};

use super::models::*;

const EOL: &'static [u8; 1] = b"\n";

macro_rules! get_data_from_response {
    ($res: ident) => {
        if $res.success() {
            match $res.data {
                Some(value) => Ok(value),
                None => Err(IoError::new(ErrorKind::Other, "value is empty"))
            }
        } else {
            Err(IoError::new(ErrorKind::Other, "mpv says command failed"))
        }
    }
}

pub struct MpvClient {
    socket: UnixStream,
}

impl MpvClient {
    pub fn new(addr: &str) -> Result<Self> {
        let stream = UnixStream::connect(addr)?;
        Ok(MpvClient { socket: stream })
    }

    pub fn load_file(&mut self, file_path: &str, mode: LoadMode) -> Result<bool> {
        let cmd = Command::LoadFile(String::from(file_path), mode);
        self.write_command_without_response(cmd)
    }

    pub fn load_list(&mut self, list_path: &str, mode: LoadMode) -> Result<bool> {
        let cmd = Command::LoadList(String::from(list_path), mode);
        self.write_command_without_response(cmd)
    }

    pub fn seek(&mut self, position: i64, mode: SeekMode) -> Result<bool> {
        let cmd = Command::Seek(position, mode);
        self.write_command_without_response(cmd)
    }

    pub fn quit(&mut self, exit_code: i64) -> Result<bool> {
        let cmd = Command::Quit(exit_code);
        self.write_command_without_response(cmd)
    }

    pub fn stop(&mut self) -> Result<bool> {
        let cmd = Command::Stop;
        self.write_command_without_response(cmd)
    }

    pub fn pause(&mut self) -> Result<bool> {
        let cmd = Command::SetProperty(String::from("pause"), Json::Boolean(true));
        self.write_command_without_response(cmd)
    }

    pub fn resume(&mut self) -> Result<bool> {
        let cmd = Command::SetProperty(String::from("pause"), Json::Boolean(false));
        self.write_command_without_response(cmd)
    }

    pub fn clear_playlist(&mut self) -> Result<bool> {
        let cmd = Command::PlaylistClear;
        self.write_command_without_response(cmd)
    }

    pub fn get_is_paused(&mut self) -> Result<bool> {
        let cmd = Command::GetProperty(String::from("pause"));
        let response = self.write_command::<bool>(cmd)?;
        get_data_from_response!(response)
    }

    pub fn get_position(&mut self) -> Result<f32> {
        let cmd = Command::GetProperty(String::from("time-pos"));
        let response = self.write_command::<f32>(cmd)?;
        get_data_from_response!(response)
    }

    pub fn get_remaining(&mut self) -> Result<f32> {
        let cmd = Command::GetProperty(String::from("time-remaining"));
        let response = self.write_command::<f32>(cmd)?;
        get_data_from_response!(response)
    }

    pub fn get_duration(&mut self) -> Result<f32> {
        let cmd = Command::GetProperty(String::from("duration"));
        let response = self.write_command::<f32>(cmd)?;
        get_data_from_response!(response)
    }

    fn check_size(size: usize) -> Result<()> {
        if size > 0 {
            Ok(())
        } else {
            Err(IoError::new(ErrorKind::WriteZero, "Failed to write command"))
        }
    }

    fn write_command_without_response(&mut self, cmd: Command) -> Result<bool> {
        let response = self.write_command::<bool>(cmd)?;
        Ok(response.success())
    }

    fn write_command<T: Decodable>(&mut self, cmd: Command) -> Result<Response<T>> {
        let request = Request::new(cmd);
        let json_content = json::encode(&request).map_err(|_| IoError::new(ErrorKind::InvalidData, "Failed to convert to json"))?;
        info!("Sending command: {}", &*json_content);
        let size = self.write(&*json_content)?;
        let _ = Self::check_size(size)?;
        self.wait_for_response::<T>()
    }

    fn write(&mut self, content: &str) -> Result<usize> {
        let content_size = self.socket.write(content.as_bytes())?;
        self.socket.write(EOL)?;
        self.socket.flush()?;
        Ok(content_size)
    }

    fn wait_for_response<T: Decodable>(&mut self) -> Result<Response<T>> {
        let mut buffer: [u8; 512] = [0 as u8; 512];
        let mut line: String = String::new();
        loop {
            let size = self.socket.read(&mut buffer)?;
            let mut start_index: usize = 0 as usize;
            loop {
                // Get the index of LF
                let line_index = Self::get_lf_index(&buffer, start_index, size).unwrap_or(size);

                // Push the current char into line
                let buffer_iter = buffer[start_index..line_index].iter().map(|&c| c as char);
                for ch in buffer_iter {
                    line.push(ch);
                }

                // If we reach the end but it's still not the LF char, continue reading
                if line_index == size && buffer[size - 1] != b'\n' {
                    break;
                }

                info!("Get response: {}", &*line);

                // Try parsing the response using the current line char
                if let Ok(res) = json::decode::<Response<T>>(&*line) {
                    return Ok(res);
                }

                if line_index == size {
                    break;
                }

                // Read the next line
                start_index = line_index + 1;
                line.clear();
            }
        }
    }

    fn get_lf_index(buffer: &[u8], start_index: usize, total_size: usize) -> Option<usize> {
        for index in start_index..total_size {
            if buffer[index] == b'\n' {
                return Some(index);
            }
        }

        None
    }
}
