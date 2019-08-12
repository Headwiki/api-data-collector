#![feature(async_await)]
mod config;

use std::fs;
use config::{Api, Config, Job};
use serde_json::{Value};

use mongodb::{bson};
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use std::thread;
use std::sync::mpsc;
use std::time::{Duration};
use chrono::prelude::*;

use futures::executor::block_on;
use futures::stream::*;

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

  let mut jobs: Vec<Job> = Vec::new();

  // Create channel where jobs will send their data to receiver(main thread)
  let (tx, rx): (mpsc::Sender<config::JobResult>, mpsc::Receiver<config::JobResult>) = mpsc::channel();

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
        //job.sender.send(job.apis[0].to_owned()).unwrap();
        loop {
          async {
          let mut running = futures::stream::FuturesUnordered::new();
          for api in job.apis.to_owned() {
            running.push(async move { get_api(&api, &job) });
          }
          while let Some(_) = running.next().await {}
          };

          thread::sleep(Duration::from_secs(job.interval))
        }
      })
    );
  }

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
fn get(url: &String) -> Result<Value, Box<std::error::Error>> {
  let resp: Value = reqwest::get(url)?
    .json()?;
  //println!("{:#?}", resp);
  Ok(resp)
}

async fn get_api<'a>(api: &'a Api, job: &'a Job) {
  let response = get(&api.url);
    match response {
      Ok(data) => {
        let job_result = config::JobResult{ api_data: data, api: api.to_owned(), time: Utc::now() };
          match job.sender.send(job_result) {
            Ok(()) => {},
            Err(e) => { eprintln!("Sender failed for: '{:?}', error: '{}'", api, e); }
          }
      },
      Err(e) => { eprintln!("Response failed for: '{:?}', error: '{}'", api, e); }
    }
}