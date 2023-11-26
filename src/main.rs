use std::{io::Write, thread, time::Duration};

mod chat;
mod secrets;

fn main() {
  // Logger setup
  env_logger::Builder::new()
    .format(|buf, record| {
      let mut style = buf.style();
      style.set_color(match record.level() {
        log::Level::Trace => env_logger::fmt::Color::Cyan,
        log::Level::Debug => env_logger::fmt::Color::Blue,
        log::Level::Info => env_logger::fmt::Color::Green,
        log::Level::Warn => env_logger::fmt::Color::Yellow,
        log::Level::Error => env_logger::fmt::Color::Red,
      });
      return writeln!(
        buf,
        "[{} {:<5}] {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        style.value(record.level()),
        record.args(),
      );
    })
    .filter_level(log::LevelFilter::Trace)
    .init();

  secrets::parse();
  chat::start();

  // Main loop?
  let sleep_dur = Duration::from_millis(10);
  loop {
    thread::sleep(sleep_dur);
  }
}
