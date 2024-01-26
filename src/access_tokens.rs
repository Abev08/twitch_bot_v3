use std::{
  io::{Read, Write},
  net::TcpListener,
  ops::Add,
  process::Command,
};

use crate::{database, secrets};

static TWITCH_SCOPE: &[&str] = &[
  "bits:read",                     // View Bits information for a channel
  "channel:manage:redemptions", // Manage Channel Points custom rewards and their redemptions on a channel
  "channel:read:hype_train",    // View Hype Train information for a channel
  "channel:read:redemptions", // View Channel Points custom rewards and their redemptions on a channel
  "channel:read:subscriptions", // View a list of all subscribers to a channel and check if a user is subscribed to a channel
  "chat:edit",                  // Send live stream chat messages
  "chat:read",                  // View live stream chat messages
  "moderator:manage:banned_users", // Ban and unban users
  "moderator:manage:shoutouts", // Manage a broadcaster’s shoutouts
  "moderator:read:chatters",    // View the chatters in a broadcaster’s chat room
  "moderator:read:followers",   // Read the followers of a broadcaster
  "whispers:edit",              // Send whisper messages
  "whispers:read",              // View your whisper messages
];

/// Updates access tokens
///
/// Returns Err() if critical error occured, otherwise Ok()
pub fn update() -> Result<(), ()> {
  log::info!("Updating access tokens");
  let mut id = String::new();
  let mut pass = String::new();
  let mut oauth = String::new();
  let mut oauth_refresh = String::new();
  // let expires: DateTime<chrono::TimeZone> = DateTime::from(database::get_data(database::Keys::TwitchExpires));

  // Twitch
  id.clear();
  id.push_str(&secrets::get_data(secrets::Keys::TwitchID));
  pass.clear();
  pass.push_str(&secrets::get_data(secrets::Keys::TwitchPassowrd));
  oauth.clear();
  oauth.push_str(&database::get_data(database::Keys::TwitchOAuth));
  oauth_refresh.clear();
  oauth_refresh.push_str(&database::get_data(database::Keys::TwitchOAuthRefresh));
  if oauth.len() == 0 || oauth_refresh.len() == 0 {
    // Get new oauth and refresh token
    twitch_get_new(&id, &pass);
  } else {
    // Update the access token
    if !twitch_refresh(&id, &pass, &oauth_refresh) {
      // Update failed, request new one
      twitch_get_new(&id, &pass);
    }
  }
  // Get channel ID
  if get_channel_id().is_err() {
    return Err(());
  }

  return Ok(());
}

fn twitch_get_new(id: &String, pass: &String) {
  let mut s = String::new();
  for i in 0..TWITCH_SCOPE.len() {
    s.push_str(TWITCH_SCOPE[i]);
    if i != TWITCH_SCOPE.len() - 1 {
      s.push_str("+");
    }
  }
  let scope = s.replace(":", "%3A"); // Change to url encoded

  let mut url = format!(
    "https://id.twitch.tv/oauth2/authorize?\
    client_id={}\
    &redirect_uri=http://localhost:3000\
    &response_type=code\
    &scope={}",
    id, scope
  );
  url = url.replace("&", "^&"); // Change to cmd encoded - the '&' symbol has to be escaped

  log::info!("Requesting user authentication for Twitch");
  if cfg!(windows) {
    Command::new("cmd.exe")
      .arg("/C")
      .arg("start")
      .arg(&url)
      .spawn()
      .expect("Something went wrong when starting new process");
  } else {
    Command::new("sh")
      .arg("-c")
      .arg(&url)
      .spawn()
      .expect("Something went wrong when starting new process");
  };

  // Start the server and wait for user reaction
  let mut code = String::new();
  let listener = TcpListener::bind("127.0.0.1:3000").unwrap();
  for stream in listener.incoming() {
    let mut buf = String::new();
    let mut connection = stream.unwrap();
    match connection.read_to_string(&mut buf) {
      Ok(_len) => {
        // println!("{}", buf);

        // Send proper response
        let contents = "<!DOCTYPE html><title>Hello, I'm in HACKERMANS</title>";
        connection
          .write_all(
            format!(
              "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
              contents.len(),
              contents
            )
            .as_bytes(),
          )
          .expect("Response was not send correctly");

        // Find 'code' part
        match buf.find("?code=") {
          Some(start) => match buf.find('&') {
            Some(end) => {
              if end > start {
                code.push_str(&buf[(start + 6)..end]); // 6 == "?code=".len()
                break;
              }
            }
            _ => {}
          },
          _ => {}
        }
      }
      _ => {}
    }
  }

  let response = ureq::post("https://id.twitch.tv/oauth2/token")
    .set("Content-Type", "application/x-www-form-urlencoded")
    .send_string(&format!("client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri=http://localhost:3000",
      id, pass, code))
    .unwrap().into_string();

  // Parse the response
  if response.is_ok() {
    log::info!("Acquired new Twitch access token");
    let resp: serde_json::Value = serde_json::from_str(&response.unwrap()).unwrap();
    let expiration = chrono::Local::now().add(chrono::Duration::seconds(
      resp["expires_in"].as_i64().unwrap(),
    ));
    database::update_value(
      database::Keys::TwitchOAuth,
      resp["access_token"].as_str().unwrap().to_string(),
    );
    database::update_value(
      database::Keys::TwitchOAuthRefresh,
      resp["refresh_token"].as_str().unwrap().to_string(),
    );
    database::update_value(database::Keys::TwitchExpires, expiration.to_string());
  } else {
    log::error!("Couldn't get new Twitch access token");
  }
}

/// Refreshes the access tokens. Returns true if new token was acquired, otherwise false.
fn twitch_refresh(id: &String, pass: &String, refresh_token: &String) -> bool {
  log::info!("Refreshing Twitch access token");
  let response = ureq::post("https://id.twitch.tv/oauth2/token")
    .set("Content-Type", "application/x-www-form-urlencoded")
    .send_string(&format!(
      "client_id={}&client_secret={}&grant_type=refresh_token&refresh_token={}",
      id,
      pass,
      refresh_token.replace(":", "%3A")
    ));

  // Parse the response
  if response.is_ok() {
    log::info!("Acquired new Twitch access token");
    let resp: serde_json::Value =
      serde_json::from_str(&response.unwrap().into_string().unwrap()).unwrap();
    let expiration = chrono::Local::now().add(chrono::Duration::seconds(
      resp["expires_in"].as_i64().unwrap(),
    ));
    database::update_value(
      database::Keys::TwitchOAuth,
      resp["access_token"].as_str().unwrap().to_string(),
    );
    database::update_value(
      database::Keys::TwitchOAuthRefresh,
      resp["refresh_token"].as_str().unwrap().to_string(),
    );
    database::update_value(database::Keys::TwitchExpires, expiration.to_string());
    return true;
  }

  log::error!(
    "Couldn't refresh Twitch access token. {}",
    response.unwrap_err()
  );
  return false;
}

/// Updates channel id from provided channel name.
fn get_channel_id() -> Result<(), ()> {
  log::info!("Requesting channel ID");

  let channel_name = &secrets::get_data(secrets::Keys::Channel);
  let twitch_id = &secrets::get_data(secrets::Keys::TwitchID);
  let twitch_oauth = &database::get_data(database::Keys::TwitchOAuth);
  let response = ureq::get(&format!(
    "https://api.twitch.tv/helix/users?login={}",
    channel_name
  ))
  .set("Authorization", &format!("Bearer {}", &twitch_oauth))
  .set("Client-Id", twitch_id)
  .call();

  if let Ok(resp) = response {
    let data: serde_json::Value = serde_json::from_str(&resp.into_string().unwrap()).unwrap();
    let data = data["data"].as_array().unwrap();
    for i in 0..data.len() {
      if data[i]["login"].as_str().unwrap() == channel_name {
        secrets::set_data(secrets::Keys::ChannelID, data[i]["id"].as_str().unwrap());
        return Ok(());
      }
    }
  }

  log::error!("Couldn't acquire broadcaster ID. Probably defined channel name doesn't exist.");
  return Err(());
}
