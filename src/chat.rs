use std::{
  collections::VecDeque,
  io::{Read, Write},
  net::TcpStream,
  sync::Mutex,
  thread::{self},
  time::{Duration, SystemTime},
};

use crate::secrets;

/// Message metadata
struct Metadata {
  badge: String,
  username: String,
  user_id: String,
  message_id: String,
  custrom_reward_id: String,
  bits: String,
}

/// PING message response
const PONG: &[u8] = b"PONG :tmi.twitch.tv\r\n";
/// Queue for messages that should be send
static SENDQUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());

/// Starts the chat bot
pub fn start() {
  log::info!("Chat bot start");

  // Create chat bot thread
  thread::Builder::new()
    .name("Chat bot".to_string())
    .spawn(move || {
      update();
    })
    .expect("Spawning chat bot thread failed!");
}

fn update() {
  let channel = secrets::CHANNEL.lock().unwrap().clone();
  if channel.len() == 0 {
    log::error!("Missing channel name");
    return;
  }
  let (mut twitch_id, mut twitch_oauth) = (String::new(), String::new());
  let mut buffer = [0u8; 16384]; // Max IRC message is 4096 bytes? let's allocate 4 times that, 2 times max message length wasn't enaugh for really fast chats
  let mut temp: usize;
  let (mut msg_start, mut msg_end): (usize, usize);
  let mut index: Option<usize>;
  let (mut msg, mut header, mut body): (&str, &str, &str);
  let mut last_send_time = SystemTime::now();
  let timeout = Some(Duration::from_millis(100));
  let timeout_error = Duration::from_secs(2);
  let mut metadata: Metadata = Metadata {
    badge: String::new(),
    username: String::new(),
    user_id: String::new(),
    message_id: String::new(),
    custrom_reward_id: String::new(),
    bits: String::new(),
  };

  loop {
    let client = TcpStream::connect("irc.chat.twitch.tv:6667");
    if client.is_err() {
      log::error!("Chat bot connection error: {}", client.unwrap_err());
      thread::sleep(timeout_error);
    } else {
      log::info!("Chat bot connected");

      twitch_id.clear();
      twitch_id.push_str(&secrets::TWITCH.lock().unwrap().id);
      twitch_oauth.clear();
      twitch_oauth.push_str(&secrets::TWITCH.lock().unwrap().pass); // FIXME: change "pass" to "token" after implementing access tokens 

      let mut stream = client.unwrap();
      stream
        .set_read_timeout(timeout)
        .expect("Something went wrong when setting read timeout");
      stream
        .write(format!("PASS oauth:{twitch_oauth}\r\n").as_bytes())
        .expect("Something went wrong when sending the message");
      stream
        .write(format!("NICK {twitch_id}\r\n").as_bytes())
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
            let data = std::str::from_utf8(&buffer[0..temp]).expect("Couldn't parse received data");

            // Loop through every message in data
            loop {
              if msg_start >= data.len() {
                break;
              }

              // Find the end of the message
              index = data[msg_start..].find("\r\n");
              msg_end = match index {
                Some(idx) => idx,
                None => data.len(),
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
                index = msg.find("PRIVMSG");
                if index.is_some() {
                  // Found the propper message, try to parse it
                  temp = index.unwrap();
                  header = &msg[..temp];
                  body = &msg[(temp + 11 + channel.len())..]; // 11 == "PRIVMSG # :".len()
                  get_metadata_data(header, &mut metadata);

                  // Print the message
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
                    println!("{:^3} {:>20}: {}", metadata.badge, metadata.username, body);

                    // Check if the chatter used some commands
                    // if body == "get system time" {
                    //   send_message_response(
                    //     &format!("{:?}", SystemTime::now()),
                    //     &metadata.message_id,
                    //   );
                    // }
                  }
                } else {
                  index = msg.find("USERNOTICE");
                  if index.is_some() {
                    // Found the propper message, try to parse it
                    temp = index.unwrap();
                    header = &msg[..temp];
                    body = &msg[(temp + 12 + channel.len())..]; // 12 == "USERNOTICE #".len()
                    get_metadata_data(header, &mut metadata);

                    // Get message type
                    let mut msg_type: &str = Default::default();
                    index = header.find("msg-id=");
                    if index.is_some() {
                      temp = index.unwrap() + 7; // 7 == "msg-id=".len()
                      index = header[temp..].find(';');
                      if index.is_some() {
                        msg_type = &header[temp..(index.unwrap() + temp)];
                      }
                    }

                    // Print the message
                    if msg_type.eq("sub") || msg_type.eq("resub") {
                      println!("> {} subscribed!{}", metadata.username, body);
                    } else if msg_type.eq("subgift") {
                      println!("> {} gifted some subs!{}", metadata.username, body);
                    } else if msg_type.eq("submysterygift") {
                      println!(
                        "> {} gifted some subs to random viewers!{}",
                        metadata.username, body
                      );
                    } else if msg_type.eq("primepaidupgrade") {
                      println!(
                        "> {} converted prime sub to standard sub!{}",
                        metadata.username, body
                      );
                    } else if msg_type.eq("announcement") {
                      println!("> {} announced that{}", metadata.username, body);
                    } else if msg_type.eq("raid") {
                      println!("> {} raided the channel!{}", metadata.username, body);
                    } else if msg_type.eq("viewermilestone") {
                      println!(
                        "> {} did something that fired viewer milestone!{}",
                        metadata.username, body
                      );
                    } else {
                      // Message type not recognized - print the whole message
                      println!("{}", msg);
                    }
                  } else {
                    index = msg.find("CLEARCHAT");
                    if index.is_some() {
                      if msg.starts_with("@ban-duration") {
                        index = msg.rfind(':');
                        if index.is_some() {
                          temp = index.unwrap() + 1;
                        } else {
                          temp = msg.len();
                        }
                        println!("> {} got banned!", &msg[temp..]);
                      }
                    } else {
                      index = msg.find("CLEARMSG");
                      if index.is_some() {
                        if msg.starts_with("@login=") {
                          index = msg.find(';');
                          if index.is_some() {
                            temp = index.unwrap();
                          } else {
                            temp = msg.len();
                          }
                          println!("> {} got perma banned!", &msg[7..temp]);
                        }
                      } else {
                        // Message separator not found, just print the message to the console
                        println!("{}", msg);
                      }
                    }
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
        if last_send_time.elapsed().unwrap() >= timeout.unwrap() {
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
    }
  }
}

fn get_metadata_data(header: &str, metadata: &mut Metadata) {
  let mut index: Option<usize>;
  let (mut temp, mut temp2): (usize, usize);

  // Get the message ID
  metadata.message_id.clear();
  index = header.find("id=");
  if index.is_some() {
    temp = index.unwrap() + 3; // 3 == "id=".len()
    index = header[temp..].find(';');
    if index.is_some() {
      temp2 = index.unwrap() + temp;
      metadata.message_id.push_str(&header[temp..temp2]);
    }
  }

  // Get the badge
  metadata.badge.clear();
  index = header.find("badges=");
  if index.is_some() {
    temp = index.unwrap() + 7; // 7 == "badges=".len()
    index = header[temp..].find(';');
    if index.is_some() {
      temp2 = index.unwrap() + temp;

      if header[temp..temp2].find("broadcaster").is_some() {
        metadata.badge.push_str("STR");
      } else if header[temp..temp2].find("moderator").is_some() {
        metadata.badge.push_str("MOD");
      } else if header[temp..temp2].find("subscriber").is_some() {
        metadata.badge.push_str("SUB");
      } else if header[temp..temp2].find("vip").is_some() {
        metadata.badge.push_str("VIP");
      }
    }
  }

  // Get the name
  metadata.username.clear();
  index = header.find("display-name=");
  if index.is_some() {
    temp = index.unwrap() + 13; // 13 == "display-name=".len()
    index = header[temp..].find(';');
    if index.is_some() {
      temp2 = index.unwrap() + temp;
      metadata.username.push_str(&header[temp..temp2]);
    }
  }

  // Get user ID
  metadata.user_id.clear();
  index = header.find("user-id=");
  if index.is_some() {
    temp = index.unwrap() + 8; // 8 == "user-id=".len()
    index = header[temp..].find(';');
    if index.is_some() {
      temp2 = index.unwrap() + temp;
      metadata.user_id.push_str(&header[temp..temp2]);
    }
  }

  // Get custom reward ID
  metadata.custrom_reward_id.clear();
  index = header.find("custom-reward-id=");
  if index.is_some() {
    temp = index.unwrap() + 17; // 17 == "custom-reward-id=".len()
    index = header[temp..].find(';');
    if index.is_some() {
      temp2 = index.unwrap() + temp;
      metadata.custrom_reward_id.push_str(&header[temp..temp2]);
    }
  }

  // Get bits amount
  metadata.bits.clear();
  index = header.find("bits=");
  if index.is_some() {
    temp = index.unwrap() + 5; // 5 == "bits=".len()
    index = header[temp..].find(';');
    if index.is_some() {
      temp2 = index.unwrap() + temp;
      metadata.bits.push_str(&header[temp..temp2]);
    }
  }
}

/// Sends provided message to the chat.
#[allow(dead_code)]
pub fn send_message(message: &String) {
  let mut msg = String::from("PRIVMSG #");
  msg.push_str(&secrets::CHANNEL.lock().unwrap());
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
  msg.push_str(&secrets::CHANNEL.lock().unwrap());
  msg.push_str(" :");
  msg.push_str(message);
  msg.push_str("\r\n");
  SENDQUEUE.lock().unwrap().push_back(msg);
}
