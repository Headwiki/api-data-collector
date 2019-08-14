#![feature(async_await)]
mod config;

use std::fs;
use config::{Api, Config};
use serde_json::{Value};

use mongodb::{bson};
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use std::thread;
use std::sync::mpsc;
use chrono::prelude::*;

use futures::executor::block_on;

fn main() {

  //tokio::run(fetch());

    // Notes for reading config file
  let contents = fs::read_to_string("config.json")
    .expect("Something went wrong reading the file");

  let config: Config =
    serde_json::from_str(&contents).expect("JSON was not well-formatted");

  //println!("{:?}", config);

     // Connect to MongoDB
  let client = mongodb::Client::connect("localhost", 27017)
    .expect("Failed to initialize standalone client.");

  // Create channel where jobs will send their data to receiver(main thread)
  let (tx, rx): (mpsc::Sender<config::ApiResult>, mpsc::Receiver<config::ApiResult>) = mpsc::channel();

/*    // Populate vector of jobs
  for api in &config.apis {  

      // Create a transmitter/sender which the job will use to send data outside of its own thread
      let new_sender = mpsc::Sender::clone(&tx);

      // Add new job
      jobs.push(Job{ interval: api.interval, sender: new_sender, apis: vec![api.to_owned()] });
  } */

  let mut cloned_apis: Vec<config::Api> = Vec::new();

  for api in &config.apis {
    cloned_apis.push(
      Api { 
        name: api.name.clone(), 
        url: api.url.clone(), 
        interval: api.interval
      }
    );
  }

  thread::spawn(move || {
    // TODO: loop over apis and execute in async fn
      block_on(async move {
        let mut running = futures::stream::FuturesUnordered::new();

        for api in cloned_apis {
          running.push( async move { start_api(api, mpsc::Sender::clone(&tx)) });
        }

        while let Some(_) = running.next().await {}
      });
  });


  // Continuously listen for data from transitters/senders
  for received in rx {
    println!("Got: {:?}", received);
      // Select collection
    let coll = client.db(&config.db).collection(&received.api.name);

    // Convert json data to bson
    let bson_data = bson::to_bson(&config::MongoData{ time: received.time, api_data: received.api_data }).unwrap();

/* 
&received.api_data
json!({"time": &received.time, data: &received.api_data}) */

    // Insert bson data into collection
    if let bson::Bson::Document(document) = bson_data {
      coll.insert_one(document, None).unwrap();  // Insert into a MongoDB collection
    } else {
      println!("Error converting the BSON object into a MongoDB document");
    };
  }  
}

// Tries to parse given url as generic json value
fn get(url: &String) -> Result<Value, Box<dyn std::error::Error>> {
  let resp: Value = reqwest::get(url)?
    .json()?;
  //println!("{:#?}", resp);
  Ok(resp)
}

async fn start_api(api: Api, sender: mpsc::Sender<config::ApiResult>) {
    let response = get(&api.url);
    match response {
      Ok(data) => {
        let api_result = config::ApiResult{ api_data: data, api: api.to_owned(), time: Utc::now() };
          match sender.send(api_result) {
            Ok(()) => {},
            Err(e) => { eprintln!("Sender failed for: '{:?}', error: '{}'", api, e); }
          }
      },
      Err(e) => { eprintln!("Response failed for: '{:?}', error: '{}'", api, e); }
    }
}