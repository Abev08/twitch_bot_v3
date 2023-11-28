I'm learning Rust.<br>
I decided that rewriting my [Twitch bot](https://github.com/Abev08/TwitchBot) in Rust could be a good learning project.<br>
Maybe one day it will be good enough to call it version 3 of the bot :)

Used dependencies:
- chrono - time => because build in time lib doesn't work for me, the time zone was incorrect,
- log, env_logger - logging => easy to use, easy to customize,
- sqlite - SQL database => a single file database is better than storing data in some file,
- ureq - http requests => easy to use, low dependency count, doesn't force me to use tokio,
- serde_json - json serialization and deserialization => easy to use, allows creating "dynamic" objects and accessing data with string keys like 'object["key"]',
