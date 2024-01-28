use std::{
  collections::VecDeque,
  net::{SocketAddr, TcpListener},
  sync::{Arc, Mutex, RwLock},
  thread,
  time::Duration,
};

use tiny_http::{Header, Response, Server, StatusCode};
use tungstenite::Message;

use crate::notifications;

const HTML_ADDRESS: &str = "127.0.0.1:40000";
const WEBSOCKET_ADDRESS: &str = "127.0.0.1:40001";

const INDEX_HTML: &str = include_str!("client/client.html");
const CLIENT_JS: &str = include_str!("client/client.js");

struct Client {
  addr: SocketAddr,
  new_msg: bool,
  queue: VecDeque<Message>,
  finished: bool,
}

impl Client {
  pub fn new(addr: SocketAddr) -> Self {
    return Client {
      addr,
      new_msg: false,
      queue: VecDeque::new(),
      finished: true,
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

  let server = Server::http(HTML_ADDRESS).unwrap();
  log::info!("Client server started at: http://{}", HTML_ADDRESS);

  for request in server.incoming_requests() {
    match request.url() {
      "/" => {
        let resp = Response::from_string(INDEX_HTML).with_header(Header {
          field: "Content-Type".parse().unwrap(),
          value: "text/html; charset=UTF-8".parse().unwrap(),
        });
        request
          .respond(resp)
          .expect("Couldn't respond to the request");
      }
      "/client.js" => {
        let resp = Response::from_string(CLIENT_JS);
        request
          .respond(resp)
          .expect("Couldn't respond to the request");
      }
      "/follow_sound" => {
        let resp = Response::from_data(notifications::DEFAULT_NOTIFICATION_SOUND);
        request
          .respond(resp)
          .expect("Couldn't respond to the request");
      }
      _ => {
        let response = Response::new_empty(StatusCode(204));
        request
          .respond(response)
          .expect("Couldn't respond to the request");
      }
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
          let msg = c.queue.pop_front();
          c.new_msg = c.queue.len() != 0;
          if let Some(m) = msg {
            let _ = websocket.send(m);
          }
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
                check_clients_finished(Some(clients));
                break;
              }
            }
          }
          return;
        }
        let msg = res.unwrap();
        println!("Message from {}, {}", client.read().unwrap().addr, msg);

        if msg.is_text() {
          let text = msg.to_text().unwrap();
          if text == "FINISHED" {
            // Notification finished client event
            client.write().unwrap().finished = true;
            check_clients_finished(None);
          }
        }
      }
    });
  }
}

pub fn send_text_message(msg: &str) -> bool {
  let clients = CONNECTED_CLIENTS.lock().unwrap();
  if clients.len() == 0 {
    return false;
  }

  for i in 0..clients.len() {
    let mut c = clients[i].write().unwrap();
    c.queue.push_back(Message::Text(msg.to_owned()));
    c.finished = false;
    c.new_msg = true;
  }
  return true;
}

fn check_clients_finished(clients: Option<std::sync::MutexGuard<'_, Vec<Arc<RwLock<Client>>>>>) {
  let _clients: std::sync::MutexGuard<'_, Vec<Arc<RwLock<Client>>>>;
  if clients.is_some() {
    _clients = clients.unwrap();
  } else {
    _clients = CONNECTED_CLIENTS.lock().unwrap();
  }

  for i in 0.._clients.len() {
    if _clients[i].read().unwrap().finished == false {
      // Early return if one of the clients didn't finish notificaiton
      return;
    }
  }

  // All of the clients finished their notifications
  notifications::NOTIFICATION_FINISHED.lock().unwrap()[0] = true;
}
