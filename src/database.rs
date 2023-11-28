use std::sync::Mutex;

use sqlite::Connection;

struct Record {
  key: Keys,
  value: String,
}

impl Record {
  pub fn new(key: Keys) -> Self {
    return Self {
      key: key,
      value: String::new(),
    };
  }
}

#[derive(Debug, PartialEq)]
pub enum Keys {
  Version,
  TwitchOAuth,
  TwitchOAuthRefresh,
  TwitchExpires,
}

static FILE: &str = ".db";
static DATA: Mutex<Vec<Record>> = Mutex::new(Vec::new());

pub fn init() {
  // Reads the current state of the key from the database or if the key is not found creates it in the database
  let mut data = DATA.lock().unwrap();
  data.push(Record::new(Keys::Version));
  data.push(Record::new(Keys::TwitchOAuth));
  data.push(Record::new(Keys::TwitchOAuthRefresh));
  data.push(Record::new(Keys::TwitchExpires));

  let connection: Connection;
  match sqlite::Connection::open(FILE) {
    Err(err) => {
      log::error!("{}", err);
      return;
    }
    Ok(conn) => {
      connection = conn;
      log::info!("Connected to database");
    }
  }

  // Try to execute some command to check if the table exists
  match connection.execute("SELECT COUNT(*) FROM Config") {
    Err(_err) => {
      log::warn!("New database detected - initializing it");
      connection.execute("CREATE TABLE Config (ID INTEGER NOT NULL UNIQUE, Name TEXT, Value TEXT, PRIMARY KEY(ID AUTOINCREMENT));").expect("Couldn't create table in the database");
    }
    _ => {}
  }

  let mut ok: bool;
  for i in 0..data.len() {
    ok = false;
    connection
      .iterate(
        format!(
          "SELECT Value FROM Config WHERE Name = '{:?}' LIMIT 1;",
          data[i].key
        ),
        |row| -> bool {
          if row[0].1.is_some() {
            data[i].value.clear();
            data[i].value.push_str(row[0].1.unwrap());
          }
          ok = true;
          return true;
        },
      )
      .expect("Something went wrong when accessing the database");
    if !ok {
      connection
        .execute(format!(
          "INSERT INTO Config (Name, Value) VALUES ('{:?}', '');",
          data[i].key
        ))
        .expect("Something went wrong when inserting data into database table");
    }

    // println!("{:?} = {}", data[i].key, data[i].value);
  }
}

#[allow(dead_code)]
pub fn get_data(key: Keys) -> String {
  let data = DATA.lock().unwrap();
  let mut ret = String::new();
  for i in 0..data.len() {
    if data[i].key == key {
      ret.push_str(&data[i].value);
    }
  }
  return ret;
}

#[allow(dead_code)]
pub fn update_value(key: Keys, value: String) {
  match sqlite::Connection::open(FILE) {
    Err(err) => {
      log::error!("{}", err);
      return;
    }
    Ok(conn) => {
      match (conn as Connection).execute(format!(
        "UPDATE Config SET Value='{}' WHERE Name='{:?}';",
        value, key
      )) {
        Err(err) => log::warn!(
          "Couldn't update the value of the key '{:?}'. Error: {:?}",
          key,
          err
        ),
        Ok(()) => {
          // Update the value in DATA array
          let mut data = DATA.lock().unwrap();
          for i in 0..data.len() {
            if data[i].key == key {
              data[i].value.clear();
              data[i].value.push_str(&value);
              break;
            }
          }
          log::info!("Updated value of key '{:?}' in the database", key);
        }
      };
    }
  }
}
