use serde_json::json;
use std::{thread, time::Duration};
use tungstenite::{client::IntoClientRequest, Message};

use crate::{database, secrets};

const WEBSOCKETURL: &str = "wss://eventsub.wss.twitch.tv/ws";
const SUBSCRIPTIONURL: &str = "https://api.twitch.tv/helix/eventsub/subscriptions";
// Test url with Twitch CLI client
// const WEBSOCKETURL: &str = "ws://127.0.0.1:8080/ws";
// const SUBSCRIPTIONURL: &str = "http://127.0.0.1:8080/eventsub/subscriptions";

pub fn start() {
  // Create events thread
  thread::Builder::new()
    .name("Events".to_string())
    .spawn(move || {
      update();
    })
    .expect("Spawning events thread failed!");
}

fn update() {
  let sleep_dur = Duration::from_millis(1000);
  let mut twitch_id = String::new();
  let mut twitch_oauth = String::new();
  let mut session_id = String::new();

  loop {
    twitch_id.clear();
    twitch_id.push_str(&secrets::get_data(secrets::Keys::TwitchID));
    twitch_oauth.clear();
    twitch_oauth.push_str(&database::get_data(database::Keys::TwitchOAuth));

    let mut request = WEBSOCKETURL.into_client_request().unwrap();
    let headers = request.headers_mut();
    headers.append("Client-Id", twitch_id.parse().unwrap());
    headers.append(
      "Authorization",
      format!("Bearer {}", &twitch_oauth).parse().unwrap(),
    );

    let res = tungstenite::connect(request);

    if res.is_ok() {
      let (mut socket, _) = res.unwrap();

      loop {
        let a = socket.read();

        if a.is_ok() {
          let message = a.unwrap();

          match message {
            Message::Ping(ping) => {
              socket
                .send(Message::Pong(ping))
                .expect("Couldn't send PONG response");
              // log::info!("Event bot: sending PONG response");
            }
            Message::Text(text) => {
              let msg: serde_json::Value = serde_json::from_str(&text).unwrap();

              if msg["metadata"]["message_type"] == "session_welcome" {
                // Eventsub welcome message
                let id = &msg["payload"]["session"]["id"];
                if id.is_string() {
                  session_id.clear();
                  session_id.push_str(id.as_str().unwrap());

                  // We have <10 sec to subscribe to an event, also another connection has to be used because we can't send messages to websocket server
                  if subscribe_to_events(&twitch_id, &twitch_oauth, &session_id) {
                    log::warn!("Events bot: every subscription failed, websocket connection would get disconnected every 10 seconds, closing events bot!");
                    return;
                  }
                } else {
                  // Something went wrong, break out of the loop and connect again
                  break;
                }
              } else if msg["metadata"]["message_type"] == "session_keepalive" {
                // Keep alive message, if it wasn't received in "keepalive_timeout_seconds" time from welcome message the connection should be restarted
                // log::info!("Event bot: got session_keepalive message");
              } else if msg["metadata"]["message_type"] == "notification" {
                // Stream notification
                let user_name: &str;
                if msg["payload"]["event"]["user_name"].is_string() {
                  user_name = msg["payload"]["event"]["user_name"].as_str().unwrap();
                } else {
                  user_name = "Anonymous";
                }

                if msg["payload"]["subscription"]["type"] == "channel.follow" {
                  // Channel follow
                  println!(">> New follow from {}.", user_name);
                } else {
                  // Unrecognized notification
                  println!("{}", msg);
                }
              } else {
                // Unrecognized message
                println!("{}", msg);
              }
            }
            _ => {
              println!("{:?}", message);
              break;
            }
          }
        } else {
          log::error!("{}", a.unwrap_err());
        }
      }
    }

    thread::sleep(sleep_dur);
  }
}

fn subscribe_to_events(twitch_id: &str, twitch_oauth: &str, session_id: &str) -> bool {
  // https://dev.twitch.tv/docs/eventsub/eventsub-subscription-types/
  let mut any_sub_succeeded = false;
  any_sub_succeeded |= subscribe("channel.follow", "2", session_id, twitch_id, twitch_oauth); // Channel got new follow
  any_sub_succeeded |= subscribe(
    "channel.subscribe",
    "1",
    session_id,
    twitch_id,
    twitch_oauth,
  ); // Channel got new subscription
  any_sub_succeeded |= subscribe(
    "channel.subscription.gift",
    "1",
    session_id,
    twitch_id,
    twitch_oauth,
  ); // Channel got gift subscription
  any_sub_succeeded |= subscribe(
    "channel.subscription.message",
    "1",
    session_id,
    twitch_id,
    twitch_oauth,
  ); // Channel got resubscription
  any_sub_succeeded |= subscribe("channel.cheer", "1", session_id, twitch_id, twitch_oauth); // Channel got cheered
  any_sub_succeeded |= subscribe(
    "channel.channel_points_custom_reward_redemption.add",
    "1",
    session_id,
    twitch_id,
    twitch_oauth,
  ); // User redeemed channel points
  any_sub_succeeded |= subscribe(
    "channel.hype_train.progress",
    "1",
    session_id,
    twitch_id,
    twitch_oauth,
  ); // A Hype Train makes progress on the specified channel

  return !any_sub_succeeded;
}

fn subscribe(
  sub_type: &str,
  version: &str,
  session_id: &str,
  twitch_id: &str,
  twitch_oauth: &str,
) -> bool {
  log::info!("Events bot subscribing to {sub_type} event.");

  let channel_id = secrets::get_data(secrets::Keys::ChannelID);
  let content = json!({
    "type": sub_type,
    "version": version,
    "condition": {
      "broadcaster_user_id": &channel_id,
      "moderator_user_id": &channel_id
    },
    "transport": {
      "method": "websocket",
      "session_id": session_id
    }
  });

  let response = ureq::post(SUBSCRIPTIONURL)
    .set("Authorization", &format!("Bearer {}", &twitch_oauth))
    .set("Client-Id", twitch_id)
    .set("Content-Type", "application/json")
    .send_string(&content.to_string());

  if let Ok(resp) = &response {
    if resp.status_text() == "Accepted" {
      return true;
    }
  }
  log::warn!("Events bot subscription failed. {}", &response.unwrap_err());
  return false;
}
