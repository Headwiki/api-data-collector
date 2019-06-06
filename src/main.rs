mod config;

use std::fs;
use config::{Config, Job};
use std::collections::HashMap;
use serde_json::{Value};

use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

use std::thread;
use std::sync::mpsc;


fn main() {

  // Notes for reading config file
  let contents = fs::read_to_string("config.json")
    .expect("Something went wrong reading the file");

  let config: Config =
    serde_json::from_str(&contents).expect("JSON was not well-formatted");

  println!("{:?}", config);

  let mut jobs: Vec<Job> = Vec::new();

  // Create channel where jobs will send their data to receiver(main thread)
  let (tx, rx): (mpsc::Sender<config::Api>, mpsc::Receiver<config::Api>) = mpsc::channel();

   // Populate vector of jobs
  'apis: for api in &config.apis {  
    for job in &mut jobs {
      if api.interval == job.interval {
        // Add to job with same interval
        job.apis.push(api.to_owned());
        break 'apis;
      }
    }
      // Create a transmitter/sender which the job will use to send data outside of its own thread
      let new_sender = mpsc::Sender::clone(&tx);

      // Add new job
      jobs.push(Job{ interval: api.interval, sender: new_sender, apis: vec![api.to_owned()] });
  }

  // Vector to hold the threads (might not be necessary)
  let mut threads: Vec<std::thread::JoinHandle<()>> = Vec::new();

  // Generate a thread per job
  for job in jobs {
    threads.push(
      thread::spawn(move || {
        // Send 'api' data to receiver (as test for now)
        job.sender.send(job.apis[0].to_owned()).unwrap();
      })
    );
  }

  // Continuously listen for data from transitters/senders
  for received in rx {
    println!("Got: {:?}", received);
  }

/*   // Connect to MongoDB
  let client = Client::connect("localhost", 27017)
    .expect("Failed to initialize standalone client.");

  
  // Select collection
  let coll = client.db("apis").collection("test");

  // Get json data from an api
  let json_data = get(&"https://httpbin.org/ip".to_owned()).unwrap();

  // Convert json data to bson
  let bson_data = bson::to_bson(&json_data).unwrap();


  // Insert bson data into collection
  if let bson::Bson::Document(document) = bson_data {
    coll.insert_one(document, None).unwrap();  // Insert into a MongoDB collection
  } else {
    println!("Error converting the BSON object into a MongoDB document");
  }; */
}

// Tries to parse given url as generic json value
fn get(url: &String) -> Result<Value, Box<std::error::Error>> {
  let resp: Value = reqwest::get(url)?
    .json()?;
  println!("{:#?}", resp);
  Ok(resp)
}