use std::{thread, time::Duration};

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
  let sleep_dur = Duration::from_millis(1000);

  loop {
    thread::sleep(sleep_dur);
  }
}
