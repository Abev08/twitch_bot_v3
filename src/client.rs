use std::{
  io::{Read, Write},
  net::TcpListener,
  thread,
  time::Duration,
};

use tungstenite::accept;

const HTML_ADDRESS: &str = "127.0.0.1:40000";
const WEBSOCKET_ADDRESS: &str = "127.0.0.1:40001";

const INDEX_HTML: &str = "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>AbevBot_v3</title><script src=\"client.js\"></script></head></html>";
const CLIENT_JS: &str = "function loaded() {
  // document.body.style.background = 'yellow';

  const para = document.createElement('p');
  para.appendChild(document.createTextNode('Test paragraph.'));
  para.style.fontSize = '72px';
  para.style.color = 'deepskyblue';
  para.style.fontFamily = 'Calibri';
  para.style.fontWeight = 'bolder';
  para.style.webkitTextStroke = '1px black';
  document.body.appendChild(para);
}

const ws = new WebSocket('ws://localhost:40001');

ws.addEventListener('open', () => {
  console.log('WebSocket connection established!');

  ws.send('Test message');
})

ws.addEventListener('message', e => {
  console.log(e);
});

window.addEventListener('load', loaded);";

pub fn start() {
  // Create client thread
  thread::Builder::new()
    .name("Clients".to_string())
    .spawn(move || {
      update_new_clients();
    })
    .expect("Spawning client handler thread failed!");

  // Create client websocket thread
  thread::Builder::new()
    .name("Clients websocekts".to_string())
    .spawn(move || update_websockets())
    .expect("Spawning client websocket thread failed!");
}

fn update_new_clients() {
  log::info!("Client server start");

  let listener = TcpListener::bind(HTML_ADDRESS).unwrap();
  let mut request_buff = [0; 1024];
  let mut request = String::new();
  let read_timeout = Some(Duration::from_millis(500));

  for stream in listener.incoming() {
    let mut connection = stream.unwrap();
    connection
      .set_read_timeout(read_timeout)
      .expect("Couldn't set read timeout");
    let res = connection.read(&mut request_buff);
    if res.is_err() {
      continue;
    }
    let n = res.unwrap();
    request.clear();
    request.push_str(std::str::from_utf8(&request_buff[..n]).unwrap());

    println!("New client connection");
    // println!("New connection: {:?}, request: {:?}", connection, request);

    if request.starts_with("GET / ") {
      connection
        .write_all(
          format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            INDEX_HTML.len(),
            INDEX_HTML
          )
          .as_bytes(),
        )
        .expect("Couldn't sent client_index.html to the conneted client");
    } else if request.starts_with("GET /client.js ") {
      connection
        .write_all(
          format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
            CLIENT_JS.len(),
            CLIENT_JS
          )
          .as_bytes(),
        )
        .expect("Couldn't sent client.js to the conneted client");
    } else {
      connection
        .write_all(b"HTTP/1.1 204 No Content")
        .expect("Couldn't sent error code to the conneted client");
    }
  }
}

fn update_websockets() {
  log::info!("Client websocket start");

  let listener = TcpListener::bind(WEBSOCKET_ADDRESS).unwrap();
  for stream in listener.incoming() {
    thread::spawn(move || {
      let mut websocket = accept(stream.unwrap()).unwrap();
      println!("New websocket connection");
      // println!("New websocket: {:?}", websocket);

      loop {
        let res = websocket.read();
        if res.is_err() {
          println!("Clocing websocket connection");
          return;
        }
        let msg = res.unwrap();
        println!("Websocket message: {msg}");

        // We do not want to send back ping/pong messages.
        if msg.is_binary() || msg.is_text() {
          websocket.send(msg).unwrap();
        }
      }
    });
  }
}
