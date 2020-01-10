#![warn(rust_2018_idioms)]

mod config;

use bytes::buf::BufExt as _;
use hyper::Client;
use std::time::Duration;
use tokio::prelude::*;
use tokio::task;
use tokio::time;

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
  // Read config file asynchronously
  let mut config_file = tokio::fs::File::open("config.json").await?;
  let mut contents = vec![];
  config_file.read_to_end(&mut contents).await?;

  // Parse config file as json
  let config: config::Config = serde_json::from_str(std::str::from_utf8(&contents).unwrap())
    .expect("JSON was not well-formatted");

  // Vec to store all spawned tasks
  let mut tasks = vec![];

  // Spawn all tasks
  for api in config.apis {
    tasks.push(task::spawn(api_collector(api)));
  }

  // Await tasks
  //  Prevents application from terminating
  //  In effect waiting tasks which will never finish
  for task in tasks {
    task.await??
  }

  Ok(())
}
async fn api_collector(api: config::Api) -> Result<()> {
  // Set how often the collector should run (in seconds)
  let mut interval = time::interval(Duration::from_secs(api.interval));
  loop {
    // Wait / Pause
    interval.tick().await;

    // Get data from api, with error handling 
    match fetch_json(api.url.parse().unwrap()).await {
      Ok(data) => {
        let api_result = config::ApiResult {
          api_data: data,
          api: api.clone(),
          time: chrono::prelude::Utc::now(),
        };
        println!("{:?}", api_result);
      }
      Err(e) => {
        eprintln!("Response failed for: '{:?}', error: '{}'", api, e);
      }
    }
  }

  // Function runs forever and will not return
  #[allow(unreachable_code)]
  Ok(())
}

// Returns json data from given url
async fn fetch_json(url: hyper::Uri) -> Result<serde_json::Value> {
  let client = Client::new();

  // Fetch the url
  let res = client.get(url).await?;

  // Asynchronously aggregate the chunks of the body
  let body = hyper::body::aggregate(res).await?;

  // Try to parse as json with serde_json
  let data = serde_json::from_reader(body.reader())?;

  Ok(data)
}

/* fn main() {

    // Read config file
  let contents = std::fs::read_to_string("config.json")
    .expect("Something went wrong reading the file");

    // Parse config file as json
  let config: config::Config =
    serde_json::from_str(&contents).expect("JSON was not well-formatted");

     // Connect to MongoDB
  let client = mongodb::Client::connect("localhost", 27017)
    .expect("Failed to initialize standalone client.");

  // Create channel where threads will send their api response data to receiver(main thread)
  let (tx, rx): (mpsc::Sender<config::ApiResult>, mpsc::Receiver<config::ApiResult>) = mpsc::channel();

  for api in config.apis.clone() {
    let sender = tx.clone();
    thread::spawn(move || {
        loop {
            let response = get(&api.url);

            match response {
              Ok(data) => {
                let api_result = config::ApiResult{
                  api_data: data,
                  api: api.clone(),
                  time: chrono::prelude::Utc::now()
                };
                match sender.send(api_result) {
                  Ok(()) => {},
                  Err(e) => { eprintln!("Sender failed for: '{:?}', error: '{}'", api, e); }
                }
              },
              Err(e) => { eprintln!("Response failed for: '{:?}', error: '{}'", api, e); }
            }

            thread::sleep(std::time::Duration::from_secs(api.interval))
        };
      });
  }


  // Continuously listen for data from transitters/senders
  for received in rx {

      // Select collection
    let coll = client.db(&config.db).collection(&received.api.name);

    // Convert json data to bson
    let bson_data = mongodb::to_bson(&config::MongoData{ time: received.time, api_data: received.api_data }).unwrap();

    // Insert bson data into collection
    if let mongodb::Bson::Document(document) = bson_data {
      coll.insert_one(document, None).unwrap();  // Insert into a MongoDB collection
    } else {
      println!("Error converting the BSON object into a MongoDB document");
    };
  }
} */

// Tries to parse given url as generic json value
/* fn get(url: &String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
  let resp: serde_json::Value = reqwest::get(url)?
    .json()?;
  Ok(resp)
} */

/* #![deny(warnings)]
#![warn(rust_2018_idioms)]

#[macro_use]
extern crate serde_derive;

use bytes::buf::BufExt as _;
use hyper::Client;

// A simple type alias so as to DRY.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> Result<()> {
    let url = "http://jsonplaceholder.typicode.com/users".parse().unwrap();
    let users = fetch_json(url).await?;
    // print users
    println!("users: {:#?}", users);

    // print the sum of ids
    let sum = users.iter().fold(0, |acc, user| acc + user.id);
    println!("sum of ids: {}", sum);
    Ok(())
}

async fn fetch_json(url: hyper::Uri) -> Result<Vec<User>> {
    let client = Client::new();

    // Fetch the url...
    let res = client.get(url).await?;

    // asynchronously aggregate the chunks of the body
    let body = hyper::body::aggregate(res).await?;

    // try to parse as json with serde_json
    let users = serde_json::from_reader(body.reader())?;

    Ok(users)
}

#[derive(Deserialize, Debug)]
struct User {
    id: i32,
    name: String,
} */

/*
TODO:
  Implement https support
  Implement save to db
  maybe look at async loop to check if its idiomatic
  check memory usage when db is implemented
 */
