use std::{collections::VecDeque, sync::Mutex, thread, time::Duration};

use serde_json::json;

use crate::{chat, client};

enum NotificationType {
  NONE,
  FOLLOW,
  SUBSCRIPTION,
  SUBSCRIPTIONEXT,
  SUBSCRIPTIONGIFT,
  SUBSCRIPTIONGIFTRECEIVED,
  BITS,
  RAID,
  CHANNELREDEMPTION,
}

struct Notification {
  thetype: NotificationType,
  message_chat: Option<String>,
  message_displayed: Option<String>,
  message_displayed_position: (i32, i32),
  message_read: Option<String>, // TTS
  played_sound: Option<String>, // name of the sound that server would be asked to provide
  played_sound_volume: f32,
  played_video: Option<String>, // name of the video that server would be asked to provide
  played_video_volume: f32,
}

impl Default for Notification {
  fn default() -> Self {
    Self {
      thetype: NotificationType::NONE,
      message_chat: None,
      message_displayed: None,
      message_displayed_position: (0, 0),
      message_read: None,
      played_sound: None,
      played_sound_volume: 1.0,
      played_video: None,
      played_video_volume: 1.0,
    }
  }
}

impl Notification {
  fn start(&self) -> bool {
    if self.message_chat.is_some() {
      let msg = self.message_chat.clone().unwrap();
      chat::send_message(&msg);
    }

    // FIXME: missing data to be sent to the client
    return client::send_text_message(
      &json!({
        "type": 1,
        "message_displayed": self.message_displayed,
        "message_displayed_position": self.message_displayed_position,
        "played_sound": self.played_sound,
        "played_sound_volume": self.played_sound_volume,
        "played_video": self.played_video,
        "played_video_volume": self.played_video_volume,
      })
      .to_string(),
    );
  }
}

pub const DEFAULT_NOTIFICATION_SOUND: &[u8] = include_bytes!("../resources/tone1.wav");
pub static NOTIFICATION_FINISHED: Mutex<[bool; 1]> = Mutex::new([false]);
static QUEUE: Mutex<VecDeque<Notification>> = Mutex::new(VecDeque::new());

pub fn start() {
  // Create notifications thread
  thread::Builder::new()
    .name("Notifications".to_string())
    .spawn(move || {
      update();
    })
    .expect("Spawning notifications thread failed!");
}

fn update() {
  let sleep_dur = Duration::from_millis(100);
  let mut started = false;
  let mut current_notificaiton: Notification;

  loop {
    if !started {
      let mut queue = QUEUE.lock().unwrap();

      if queue.len() > 0 {
        current_notificaiton = queue.pop_front().unwrap();
        if current_notificaiton.start() {
          // .start() returned true - there are some clients playing notificaiton
          started = true;
          NOTIFICATION_FINISHED.lock().unwrap()[0] = false;
        }
      }
    } else {
      if NOTIFICATION_FINISHED.lock().unwrap()[0] {
        started = false; // Reset started flag to start next notification
      }
    }

    thread::sleep(sleep_dur);
  }
}

pub fn add_follow_notification(user_name: &str) {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::FOLLOW,
    message_chat: Some(format!("@{} thank you for following!", user_name)),
    message_displayed: Some(format!("New follower {}!", user_name)),
    message_displayed_position: (100, 200),
    played_sound: Some("follow_sound".to_owned()),
    played_sound_volume: 0.2,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_subscription_notification() {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::SUBSCRIPTION,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_subscription_ext_notification() {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::SUBSCRIPTIONEXT,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_subscription_gift_notification() {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::SUBSCRIPTIONGIFT,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_subscription_gift_received_notification() {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::SUBSCRIPTIONGIFTRECEIVED,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_bits_notification() {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::BITS,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_raid_notification() {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::RAID,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_channel_redemption_notification() {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::CHANNELREDEMPTION,
    ..Default::default()
  };
  queue.push_back(notification);
}
