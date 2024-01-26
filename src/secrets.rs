use std::{
  fs::File,
  io::{BufRead, BufReader, Write},
  path::Path,
  sync::Mutex,
};

struct Record {
  key: Keys,
  value: String,
}

impl Record {
  pub fn new(key: Keys) -> Self {
    return Self {
      key: key,
      value: String::new(),
    };
  }
}

#[derive(Debug, PartialEq)]
pub enum Keys {
  Channel,
  ChannelID,
  TwitchName,
  TwitchID,
  TwitchPassowrd,
}

static FILE: &str = "secrets.ini";
static DATA: Mutex<Vec<Record>> = Mutex::new(Vec::new());

/// Parses secrets.ini file
pub fn parse() -> Result<(),()> {
  log::info!("Parsing secrets file");
  let mut data = DATA.lock().unwrap();
  data.push(Record::new(Keys::Channel));
  data.push(Record::new(Keys::ChannelID));
  data.push(Record::new(Keys::TwitchID));
  data.push(Record::new(Keys::TwitchName));
  data.push(Record::new(Keys::TwitchPassowrd));

  let (mut key, mut value): (&str, &str);
  let mut index: usize;

  let file = Path::new(FILE);
  if !file.exists() {
    create_file();
    log::warn!("New secrets.ini file created. Fill it up and restart the bot");
    return Err(());
  }

  let mut required_info = RequiredInfo::new();
  let file = File::open(file);
  if file.is_ok() {
    let f = BufReader::new(file.unwrap());
    for line in f.lines() {
      let l = line.unwrap();
      if l.starts_with("//") || l.starts_with('#') {
        continue; // Skip commented out lines
      }

      match l.find('=') {
        Some(idx) => index = idx,
        None => continue,
      };
      key = l[..index].trim();
      value = &l[(index + 1)..];

      // Remove inline comments
      match value.find("//") {
        Some(idx) => value = &value[..idx],
        None => {}
      }
      match value.find('#') {
        Some(idx) => value = &value[..idx],
        None => {}
      }
      value = value.trim();

      if key == format!("{:?}", Keys::Channel) {
        let temp = &value.to_lowercase();
        _set_data(&mut data, Keys::Channel, temp);
        required_info.channel.clear();
        required_info.channel.push_str(temp);
      } else if key == format!("{:?}", Keys::TwitchName) {
        _set_data(&mut data, Keys::TwitchName, value);
        required_info.twitch_name.clear();
        required_info.twitch_name.push_str(value);
      } else if key == format!("{:?}", Keys::TwitchID) {
        _set_data(&mut data, Keys::TwitchID, value);
        required_info.twitch_id.clear();
        required_info.twitch_id.push_str(value);
      } else if key == format!("{:?}", Keys::TwitchPassowrd) {
        _set_data(&mut data, Keys::TwitchPassowrd, value);
        required_info.twitch_pass.clear();
        required_info.twitch_pass.push_str(value);
      }
    }
  } else {
    log::error!("{}", file.unwrap_err());
    return Err(());
  }

  // Check if all required info is provided
  if !required_info.filled_up() {
    log::warn!("Missing required information in secrets.ini file");
    return Err(());
  }
  return Ok(());
}

fn _set_data(data: &mut Vec<Record>, key: Keys, value: &str) {
  for i in 0..data.len() {
    if data[i].key == key {
      data[i].value.clear();
      data[i].value.push_str(value);
      return;
    }
  }
}

pub fn set_data(key: Keys, value: &str) {
  let mut data = DATA.lock().unwrap();
  for i in 0..data.len() {
    if data[i].key == key {
      data[i].value.clear();
      data[i].value.push_str(value);
      return;
    }
  }
}

pub fn get_data(key: Keys) -> String {
  let data = DATA.lock().unwrap();
  let mut ret = String::new();
  for i in 0..data.len() {
    if data[i].key == key {
      ret.push_str(&data[i].value);
      break;
    }
  }
  return ret;
}

fn create_file() {
  log::info!("Creating new secrets file");
  let new_file = File::create(FILE);
  if new_file.is_ok() {
    let mut content = String::new();
    content.push_str(&format!("{:?} = \n", Keys::Channel));
    content.push_str("\n");
    content.push_str(&format!("{:?} = \n", Keys::TwitchName));
    content.push_str(&format!("{:?} = \n", Keys::TwitchID));
    content.push_str(&format!("{:?} = \n", Keys::TwitchPassowrd));

    new_file
      .unwrap()
      .write(content.as_bytes())
      .expect("Something went wrong when writing to the file");
  }
}

struct RequiredInfo {
  channel: String,
  twitch_name: String,
  twitch_id: String,
  twitch_pass: String,
}

impl RequiredInfo {
  fn new() -> Self {
    Self {
      channel: String::new(),
      twitch_name: String::new(),
      twitch_id: String::new(),
      twitch_pass: String::new(),
    }
  }

  fn filled_up(&self) -> bool {
    return self.channel.len() > 0
      && self.twitch_name.len() > 0
      && self.twitch_id.len() > 0
      && self.twitch_pass.len() > 0;
  }
}
