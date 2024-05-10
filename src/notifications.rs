use std::{collections::VecDeque, sync::Mutex, thread, time::Duration};

use serde_json::json;

use crate::{chat, client};

#[derive(Copy, Clone)]
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

impl NotificationType {
  pub fn index(&self) -> usize {
    *self as usize
  }
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
  played_video_position: (i32, i32),
  played_video_size: (i32, i32),
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
      played_video_position: (0, 0),
      played_video_size: (0, 0),
    }
  }
}

impl Notification {
  /// Starts this notification.
  fn start(&self) -> bool {
    if self.message_chat.is_some() {
      let msg = self.message_chat.clone().unwrap();
      chat::send_message(&msg);
    }

    // FIXME: missing data to be sent to the client
    return client::send_text_message(
      &json!({
        "type": self.thetype.index(),

        "message_displayed": self.message_displayed,
        "message_displayed_position": self.message_displayed_position,

        "played_sound": self.played_sound,
        "played_sound_volume": self.played_sound_volume,

        "played_video": self.played_video,
        "played_video_volume": self.played_video_volume,
        "played_video_position": self.played_video_position,
        "played_video_size": self.played_video_size,
      })
      .to_string(),
    );
  }

  /// Plays next step of this notification.
  fn next_step(&self) -> bool {
    // TODO: Notification could depend on action queue.
    // A notification could have multiple actions like:
    // - send chat message,
    // - play sound,
    // - play video,
    // - etc.
    // The action queue could play each action in sequence.
    // It would be good for notification configurations.
    // Also each action could have finished flag
    // and actions could wait for other actions to finish.
    return true;
  }
}

pub const DEFAULT_NOTIFICATION_SOUND: &[u8] = include_bytes!("../resources/tone1.wav");
pub const DEFAULT_SUB_VIDEO: &[u8] = include_bytes!("../resources/peepoHey.mp4");

pub static NOTIFICATION_FINISHED: Mutex<[bool; 1]> = Mutex::new([false]);
/// Currently queued notifications.
static QUEUE: Mutex<VecDeque<Notification>> = Mutex::new(VecDeque::new());
/// Previously played notifications.
static PREVIOUS_NOTIFICATIONS: Mutex<VecDeque<Notification>> = Mutex::new(VecDeque::new());

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

        // Add the notification to previously played
        let mut prev = PREVIOUS_NOTIFICATIONS.lock().unwrap();
        prev.push_back(current_notificaiton);
        while prev.len() > 20 {
          prev.pop_front();
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
    played_sound: Some("follow_sound".to_string()),
    played_sound_volume: 0.2,
    ..Default::default()
  };
  queue.push_back(notification);
}

pub fn add_subscription_notification(user_name: &str) {
  let mut queue = QUEUE.lock().unwrap();
  let notification = Notification {
    thetype: NotificationType::SUBSCRIPTION,
    message_displayed: Some(format!("{} just subscribed!", user_name)),
    message_displayed_position: (100, 200),
    played_video: Some("sub_video".to_string()),
    played_video_volume: 0.5,
    played_video_position: (100, 400),
    played_video_size: (200, 200),
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
