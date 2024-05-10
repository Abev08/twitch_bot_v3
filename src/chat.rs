use std::{
  collections::VecDeque,
  io::{Read, Write},
  net::TcpStream,
  sync::Mutex,
  thread::{self},
  time::{Duration, SystemTime},
};

use crate::{database, secrets};

/// Message metadata
struct Metadata {
  message_type: String,
  badge: String,
  username: String,
  user_id: String,
  message_id: String,
  custrom_reward_id: String,
  bits: String,
  msg_id: String,
}

impl Metadata {
  fn clear(&mut self) {
    self.message_type.clear();
    self.badge.clear();
    self.username.clear();
    self.user_id.clear();
    self.message_id.clear();
    self.custrom_reward_id.clear();
    self.bits.clear();
    self.msg_id.clear();
  }
}

/// Should chat messages be printed to console window?
const PRINT_CHAT_MESSAGES: bool = false;
/// PING message response
const PONG: &[u8] = b"PONG :tmi.twitch.tv\r\n";
/// Queue for messages that should be send
static SENDQUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
/// Minimum time between sending messages
const SEND_TIMEOUT: Duration = Duration::from_millis(100);

/// Starts the chat bot
pub fn start() {
  log::info!("Chat bot start");

  // Create chat bot thread
  thread::Builder::new()
    .name("Chat".to_string())
    .spawn(move || {
      update();
    })
    .expect("Spawning chat bot thread failed!");
}

fn update() {
  let channel = secrets::get_data(secrets::Keys::Channel);
  if channel.len() == 0 {
    log::error!("Missing channel name");
    return;
  }
  let (mut twitch_name, mut twitch_oauth) = (String::new(), String::new());
  let mut buffer = [0u8; 16384]; // Max IRC message is 4096 bytes? let's allocate 4 times that, 2 times max message length wasn't enaugh for really fast chats
  let mut temp: usize;
  let (mut msg_start, mut msg_end): (usize, usize);
  let mut index: Option<usize>;
  let (mut msg, mut header, mut body): (&str, &str, &str);
  let mut last_send_time = SystemTime::now();
  let timeout_error = Duration::from_secs(2);
  let mut metadata: Metadata = Metadata {
    message_type: String::new(),
    badge: String::new(),
    username: String::new(),
    user_id: String::new(),
    message_id: String::new(),
    custrom_reward_id: String::new(),
    bits: String::new(),
    msg_id: String::new(),
  };
  let mut data = String::new();

  loop {
    let client = TcpStream::connect("irc.chat.twitch.tv:6667");
    if client.is_err() {
      log::error!("Chat bot connection error: {}", client.unwrap_err());
      thread::sleep(timeout_error);
    } else {
      log::info!("Chat bot connected");

      twitch_name.clear();
      twitch_name.push_str(&secrets::get_data(secrets::Keys::TwitchName));
      twitch_oauth.clear();
      twitch_oauth.push_str(&database::get_data(database::Keys::TwitchOAuth));

      let mut stream = client.unwrap();
      stream
        .set_read_timeout(Some(SEND_TIMEOUT))
        .expect("Something went wrong when setting read timeout");
      stream
        .write(format!("PASS oauth:{twitch_oauth}\r\n").as_bytes())
        .expect("Something went wrong when sending the message");
      stream
        .write(format!("NICK {twitch_name}\r\n").as_bytes())
        .expect("Something went wrong when sending the message");
      stream
        .write(format!("JOIN #{channel},#{channel}\r\n").as_bytes())
        .expect("Something went wrong when sending the message");
      stream
        .write("CAP REQ :twitch.tv/commands twitch.tv/tags\r\n".as_bytes())
        .expect("Something went wrong when sending the message");

      loop {
        // Receive
        let res = stream.read(&mut buffer);
        if res.is_ok() {
          temp = res.unwrap();
          if temp > 0 {
            msg_start = 0;
            // data is whole received message, it may contain multiple messages
            let d = String::from_utf8_lossy(&buffer[..temp]).into_owned(); // lossy conversion is needed because if the data contained a char outside of utf-8 range it will crash the program
            data.push_str(&d);
            drop(d);

            // Loop through every message in data
            loop {
              if msg_start >= data.len() {
                data.clear();
                break;
              }

              // Find the end of the message
              index = data[msg_start..].find("\r\n");
              msg_end = match index {
                Some(idx) => idx,
                None => {
                  let temp = String::from(&data[msg_start..]);
                  data.clear();
                  data.push_str(&temp);
                  break;
                }
              } + msg_start;
              if msg_end > data.len() {
                msg_end = data.len();
              }

              // Get the message
              msg = &data[msg_start..msg_end];

              if msg.starts_with("PING") {
                stream
                  .write(PONG)
                  .expect("Something went wrong when sending the message");
              } else {
                (header, body) = parse_message(&msg, &mut metadata);

                match metadata.message_type.as_str() {
                  "PRIVMSG" => {
                    if metadata.custrom_reward_id.len() > 0 {
                      println!(
                        "> {} redeemed custom reward with ID: {}. {}",
                        metadata.username, metadata.custrom_reward_id, body
                      );
                    } else if metadata.bits.len() > 0 {
                      println!(
                        "> {} cheered with {} bits. {}",
                        metadata.username, metadata.bits, body
                      );
                    } else {
                      if PRINT_CHAT_MESSAGES {
                        println!("{:^3} {:>20}: {}", metadata.badge, metadata.username, body);
                      }
                      check_for_commands(&metadata, body);
                    }
                  }
                  "USERNOTICE" => {
                    match metadata.msg_id.as_str() {
                      "sub" => {
                        println!("> {} subscribed! {}", metadata.username, body);
                      }
                      "resub" => {
                        println!("> {} resubscribed! {}", metadata.username, body);
                      }
                      "subgift" => {
                        let mut receipent = "";
                        index = header.find("msg-param-recipient-display-name=");
                        if index.is_some() {
                          temp = index.unwrap() + 33; // 33 == "msg-param-recipient-display-name=".len()
                          index = header[temp..].find(';');
                          if index.is_some() {
                            receipent = &header[temp..(index.unwrap() + temp)];
                          }
                        }
                        println!(
                          "> {} gifted sub to {}! {}",
                          metadata.username, receipent, body
                        );
                      }
                      "submysterygift" => {
                        println!(
                          "> {} gifted some subs to random viewers! {}",
                          metadata.username, body
                        );
                      }
                      "primepaidupgrade" => {
                        println!(
                          "> {} converted prime sub to standard sub! {}",
                          metadata.username, body
                        );
                      }
                      "giftpaidupgrade" => {
                        println!(
                          "> {} continuing sub gifted by another chatter! {}",
                          metadata.username, body
                        );
                      }
                      "communitypayforward" => {
                        println!(
                          "> {} is paying forward sub gifted by another chatter! {}",
                          metadata.username, body
                        );
                      }
                      "announcement" => {
                        println!("> {} announced that {}", metadata.username, body);
                      }
                      "raid" => {
                        println!("> {} raided the channel! {}", metadata.username, body);
                      }
                      "viewermilestone" => {
                        println!(
                          "> {} did something that fired viewer milestone! {}",
                          metadata.username, body
                        );
                      }
                      _ => {
                        // Message type not recognized - print the whole message
                        println!("{}", msg);
                      }
                    }
                  }
                  "CLEARCHAT" => {
                    if msg.starts_with("@ban-duration") {
                      index = msg.rfind(':');
                      if index.is_some() {
                        temp = index.unwrap() + 1;
                      } else {
                        temp = msg.len();
                      }
                      println!("> {} got banned!", &msg[temp..]);
                    } else if body.len() > 0 {
                      println!("> {} chat messages got cleared", &body);
                    } else {
                      println!("> Chat got cleared");
                    }
                  }
                  "CLEARMSG" => {
                    if msg.starts_with("@login=") {
                      index = msg.find(';');
                      if index.is_some() {
                        temp = index.unwrap();
                      } else {
                        temp = msg.len();
                      }
                      println!("> {} got perma banned!", &msg[7..temp]);
                    } else {
                      println!("> Someones messages got cleared")
                    }
                  }
                  "NOTICE" => {
                    match metadata.msg_id.as_str() {
                      "emote_only_on" => {
                        println!("> This room is now in emote-only mode.");
                      }
                      "emote_only_off" => {
                        println!("> This room is no longer in emote-only mode.");
                      }
                      "subs_on" => {
                        println!("> This room is now in subscribers-only mode.");
                      }
                      "subs_off" => {
                        println!("> This room is no longer in subscribers-only mode.");
                      }
                      "followers_on" | "followers_on_zero" => {
                        println!("> This room is now in followers-only mode.");
                      }
                      "followers_off" => {
                        println!("> This room is no longer in followers-only mode.");
                      }
                      "slow_on" => {
                        println!("> This room is now in slow mode.");
                      }
                      "slow_off" => {
                        println!("> This room is no longer in slow mode.");
                      }
                      _ => {
                        // Message type not recognized - print the whole message
                        println!("{}", msg);
                      }
                    }
                  }
                  "ROOMSTATE" => {
                    // Room state changed - do nothing? This message is always send with another one?
                  }
                  "USERSTATE" => {
                    if PRINT_CHAT_MESSAGES {
                      // Bot message
                      println!("> Bot message from {}", metadata.username);
                    }
                  }
                  _ => {
                    // Not recognized message
                    println!("{}", msg);
                  }
                }
              }

              // Move the start index of the message to the end index + 2 characters ("\r\n")
              msg_start = msg_end + 2;
            }
          } else {
            log::warn!("Chat bot connection was closed due to receiving zero-length data. Waiting some time and reconnecting");
            let _ = stream.shutdown(std::net::Shutdown::Both);
            thread::sleep(timeout_error);
            break;
          }
        }

        // Send
        match last_send_time.elapsed() {
          Ok(elapsed) => {
            if elapsed >= SEND_TIMEOUT {
              last_send_time = SystemTime::now();

              let mut queue = SENDQUEUE.lock().unwrap();
              if queue.len() > 0 {
                let msg = queue.pop_front().unwrap();
                stream
                  .write(msg.as_bytes())
                  .expect("Something went wrong when sending the message");
                drop(msg);
              }
            }
          }
          Err(err) => {
            log::error!(
              "Error when calculating elapsed time since last message: {}",
              err
            );
          }
        }
      }
    }
  }
}

/// Parses provided `msg` returning `header` and `body` part of it and updating it's `metadata`
fn parse_message<'a>(msg: &'a str, metadata: &mut Metadata) -> (&'a str, &'a str) {
  metadata.clear();

  let (mut temp, mut temp2): (usize, usize);
  let (header, body): (&str, &str);

  // Find header <-> body "separator"
  match msg.find("tmi.twitch.tv") {
    Some(index) => temp = index,
    None => {
      log::warn!("Chat message not parsed correctly\n{}", msg);
      return ("", "");
    }
  }

  // Get message header
  header = &msg[..temp];
  temp += 14; // 14 == "tmi.twitch.tv ".len()

  // Get message type
  temp2 = match msg[temp..].find(' ') {
    Some(index) => index,
    None => msg.len() - temp, // Reconnect message - ":tmi.twitch.tv RECONNECT", maybe something more?
  };
  metadata.message_type.push_str(&msg[temp..(temp + temp2)]);
  temp2 += 1 + temp;

  // Get message body
  body = match msg[temp2..].find(':') {
    Some(index) => &msg[(index + 1 + temp2)..],
    None => "", // No message body found
  };

  // Get header data
  let header_data: Vec<&str> = header.split(&[';', ' '][..]).collect();
  for data in header_data {
    if data.starts_with("id=") {
      // Message ID
      metadata.message_id.push_str(&data[3..]); // 3 == "id=".len()
    } else if data.starts_with("badges=") {
      // Badge
      let badge = &data[7..]; // 7 == "badges=".len()
      if badge.starts_with("broadcaster") {
        metadata.badge.push_str("STR");
      } else if badge.starts_with("moderator") {
        metadata.badge.push_str("MOD");
      } else if badge.starts_with("subscriber") {
        metadata.badge.push_str("SUB");
      } else if badge.starts_with("vip") {
        metadata.badge.push_str("VIP");
      }
    } else if data.starts_with("display-name=") {
      // Chatter name
      metadata.username.push_str(&data[13..]); // 13 == "display-name=".len()
    } else if data.starts_with("user-id=") {
      // Chatter user ID
      metadata.user_id.push_str(&data[8..]); // 8 == "user-id=".len()
    } else if data.starts_with("custom-reward-id=") {
      // Custom reward ID
      metadata.custrom_reward_id.push_str(&data[17..]); // 17 == "custom-reward-id=".len()
    } else if data.starts_with("bits=") {
      // Bits
      metadata.bits.push_str(&data[5..]); // 5 == "bits=".len()
    } else if data.starts_with("@msg-id=") {
      // msg_id - special message type, being it's own message
      metadata.msg_id.push_str(&data[8..]); // 8 == "@msg-id=".len()
    } else if data.starts_with("msg-id=") {
      // msg_id - special message type, attached to normal message
      metadata.msg_id.push_str(&data[7..]); // 7 == "msg-id=".len()
    }
  }

  return (header, body);
}

/// Sends provided message to the chat.
pub fn send_message(message: &String) {
  let mut msg = String::from("PRIVMSG #");
  msg.push_str(&secrets::get_data(secrets::Keys::Channel));
  msg.push_str(" :");
  msg.push_str(message);
  msg.push_str("\r\n");
  SENDQUEUE.lock().unwrap().push_back(msg);
}

/// Sends provided message to the chat as response to provided message id.
#[allow(dead_code)]
pub fn send_message_response(message: &String, message_id: &String) {
  let mut msg = String::from("@reply-parent-msg-id=");
  msg.push_str(message_id);
  msg.push_str(" PRIVMSG #");
  msg.push_str(&secrets::get_data(secrets::Keys::Channel));
  msg.push_str(" :");
  msg.push_str(message);
  msg.push_str("\r\n");
  SENDQUEUE.lock().unwrap().push_back(msg);
}

fn check_for_commands(metadata: &Metadata, msg: &str) {
  match msg {
    "!bot" => {
      send_message_response(&"The bot is under development, you can check it out at https://github.com/Abev08/twitch_bot_v3".to_string(), &metadata.message_id);
    }
    // "get system time" => {
    //   send_message_response(&format!("{:?}", SystemTime::now()), &metadata.message_id);
    // }
    // "!example" => {
    //   send_message_response(&"Example response".to_owned(), &metadata.message_id);
    // }
    _ => {}
  }
}
