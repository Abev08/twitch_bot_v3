use std::{
  fs::File,
  io::{BufRead, BufReader, Write},
  path::Path,
  sync::Mutex,
};

static FILE: &str = "secrets.ini";

pub struct OAuthStuff {
  pub id: String,
  pub pass: String,
  pub token: String,
  pub refresh_token: String,
}

pub static TWITCH: Mutex<OAuthStuff> = Mutex::new(OAuthStuff {
  id: String::new(),
  pass: String::new(),
  token: String::new(),
  refresh_token: String::new(),
});

pub static CHANNEL: Mutex<String> = Mutex::new(String::new());

pub fn parse() {
  log::info!("Parsing secrets file");
  let (mut key, mut value): (&str, &str);
  let mut index: usize;

  let file = Path::new(FILE);
  if !file.exists() {
    create_file();
  }

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

      if key == "CHANNEL" {
        let temp = value.to_lowercase();
        CHANNEL.lock().unwrap().push_str(temp.as_str());
      } else if key == "TWITCH_ID" {
        TWITCH.lock().unwrap().id.push_str(value);
      } else if key == "TWITCH_PASSWORD" {
        TWITCH.lock().unwrap().pass.push_str(value);
      }
    }
  } else {
    log::error!("{}", file.unwrap_err());
    return;
  }
}

fn create_file() {
  log::info!("Creating new secrets file");
  let new_file = File::create(FILE);
  if new_file.is_ok() {
    let mut content = String::new();
    content.push_str("CHANNEL = \n");
    content.push_str("\n");
    content.push_str("TWITCH_ID = \n");
    content.push_str("TWITCH_PASSWORD = \n");

    new_file
      .unwrap()
      .write(content.as_bytes())
      .expect("Something went wrong when writing to the file");
  }
}
