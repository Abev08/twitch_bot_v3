use std::sync::Mutex;

use sqlite::Connection;

pub struct DbRecord {
  key: Keys,
  value: String,
}

impl DbRecord {
  pub fn new(key: Keys) -> Self {
    Self {
      key: key,
      value: String::new(),
    }
  }
}

#[derive(Debug, PartialEq)]
pub enum Keys {
  Version,
  TwitchOAuth,
  TwitchOAuthRefresh,
}

static FILE: &str = ".db";
static DATA: Mutex<Vec<DbRecord>> = Mutex::new(Vec::new());

pub fn init() {
  let connection: Connection;
  match sqlite::Connection::open(FILE) {
    Err(err) => {
      log::error!("{}", err);
      return;
    }
    Ok(con) => {
      connection = con;
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

  // Reads the current state of the key from the database or if the key is not found creates it in the database
  let mut secrets = DATA.lock().unwrap();
  secrets.push(DbRecord::new(Keys::Version));
  secrets.push(DbRecord::new(Keys::TwitchOAuth));
  secrets.push(DbRecord::new(Keys::TwitchOAuthRefresh));

  let mut ok: bool;
  for i in 0..secrets.len() {
    ok = false;
    connection
      .iterate(
        format!(
          "SELECT Value FROM Config WHERE Name = '{:?}' LIMIT 1;",
          secrets[i].key
        ),
        |row| -> bool {
          if row[0].1.is_some() {
            secrets[i].value.clear();
            secrets[i].value.push_str(row[0].1.unwrap());
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
          secrets[i].key
        ))
        .expect("Something went wrong when inserting data into database table");
    }

    // println!("{:?} = {}", secrets[i].key, secrets[i].value);
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
