use std::{io::Write, thread, time::Duration};

mod access_tokens;
mod chat;
mod client;
mod database;
mod events;
mod notifications;
mod secrets;

fn main() {
  // Logger setup
  env_logger::Builder::new()
    .format(|buf, record| {
      let style = buf.default_level_style(record.level());
      return writeln!(
        buf,
        "[{} {}{}{}] {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        style.render(),
        record.level(),
        style.render_reset(),
        record.args(),
      );
    })
    .filter_level(log::LevelFilter::Info)
    .init();

  log::info!("Hi! I'm AbevBot v3 :)");

  database::init();
  if secrets::parse().is_err() {
    return;
  }
  if access_tokens::update().is_err() {
    return;
  }

  chat::start();
  events::start();

  client::start();
  notifications::start();

  // Main loop?
  let sleep_dur = Duration::from_millis(10);
  loop {
    thread::sleep(sleep_dur);
  }
}
