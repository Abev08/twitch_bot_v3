use std::{
  io::{Read, Write},
  net::{SocketAddr, TcpListener},
  sync::{Arc, Mutex, RwLock},
  thread,
  time::Duration,
};

use tungstenite::Message;

const HTML_ADDRESS: &str = "127.0.0.1:40000";
const WEBSOCKET_ADDRESS: &str = "127.0.0.1:40001";

const INDEX_HTML: &str = include_str!("client/client.html");
const CLIENT_JS: &str = include_str!("client/client.js");

struct Client {
  addr: SocketAddr,
  new_msg: bool,
  msg: String,
}

impl Client {
  pub fn new(addr: SocketAddr) -> Self {
    return Client {
      addr,
      new_msg: false,
      msg: String::new(),
    };
  }
}

static CONNECTED_CLIENTS: Mutex<Vec<Arc<RwLock<Client>>>> = Mutex::new(Vec::new());

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

  log::info!("Client server started at: {}", HTML_ADDRESS);

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

    // Return stuff based on request url
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
        .expect("Couldn't sent client.html to the conneted client");
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
  let read_timeout = Some(Duration::from_millis(10));
  for stream in listener.incoming() {
    thread::spawn(move || {
      let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();
      websocket
        .get_mut()
        .set_read_timeout(read_timeout)
        .expect("Couldn't set read timeout for the websocket!");

      let client = Arc::new(RwLock::new(Client::new(
        websocket.get_ref().peer_addr().unwrap(),
      )));
      {
        CONNECTED_CLIENTS.lock().unwrap().push(client.clone());
      }

      println!("New connection: {}", client.read().unwrap().addr);

      loop {
        // Send messages
        if client.read().unwrap().new_msg {
          let mut c = client.write().unwrap();
          c.new_msg = false;

          let _ = websocket.send(Message::text(&c.msg));
        }

        // Read messages
        let res = websocket.read();
        if res.is_err() {
          let err = res.unwrap_err();
          if let tungstenite::Error::Io(_) = err {
            continue;
          } else {
            println!(
              "Websocket {} connection error: {}",
              client.read().unwrap().addr,
              err
            );
          }
          // Delete dropped client from connected clients vec
          {
            let mut clients = CONNECTED_CLIENTS.lock().unwrap();
            for i in 0..clients.len() {
              if clients[i].read().unwrap().addr == client.read().unwrap().addr {
                println!("Dropped connection: {}", client.read().unwrap().addr);
                clients.remove(i);
                break;
              }
            }
          }
          return;
        }
        let msg = res.unwrap();
        println!("Message from {}, {}", client.read().unwrap().addr, msg);

        // We do not want to send back ping/pong messages.
        if msg.is_binary() || msg.is_text() {
          websocket.send(msg).unwrap();
        }
      }
    });
  }
}

pub fn send_text_message(msg: &str) {
  let clients = CONNECTED_CLIENTS.lock().unwrap();
  for i in 0..clients.len() {
    let mut c = clients[i].write().unwrap();
    c.new_msg = true;
    c.msg.clear();
    c.msg.push_str(msg);
  }
}
