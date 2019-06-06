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
  //println!("{:?}", get(&config.apis[0].url).unwrap());

  let mut jobs: Vec<Job> = Vec::new();
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
      let new_sender = mpsc::Sender::clone(&tx);
      // Add new job with new interval
      jobs.push(Job{ interval: api.interval, sender: new_sender, apis: vec![api.to_owned()] });
  }


  //println!("{:?}", jobs);


  let mut threads: Vec<std::thread::JoinHandle<()>> = Vec::new();

  for job in jobs {
    threads.push(
      thread::spawn(move || {
        //println!("Got interval of: {}", job.interval);
        job.sender.send(job.apis[0].to_owned()).unwrap();
      })
    );
  }

  /* for handle in threads {
    handle.join().unwrap();
  } */
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