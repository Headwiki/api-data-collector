mod config;

use std::thread;
use std::sync::mpsc;

use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

fn main() {

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
}

// Tries to parse given url as generic json value
fn get(url: &String) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
  let resp: serde_json::Value = reqwest::get(url)?
    .json()?;
  Ok(resp)
}