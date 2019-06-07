mod config;

use std::fs;
use config::{Config, Job};
use std::collections::HashMap;
use serde_json::{Value};

use mongodb::{Bson, bson, doc};
use mongodb::ThreadedClient;
use mongodb::db::ThreadedDatabase;

use std::thread;
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime};
use chrono::prelude::*;

use futures::Future;
use reqwest::r#async::{Client, Response};

use futures::Stream;
use futures::future::ok;
use futures::stream::iter_ok;

/* fn fetch() -> impl Future<Item=(), Error=()> {
    let client = Client::new();

    let json = |mut res : Response | {
        res.json::<Value>()
    };

    let mut requests = Vec::new();

    for _i in 0..2 {
      requests.push(client
            .get("https://httpbin.org/ip")
            .send()
            .and_then(json));
    }


     for res in requests {
          res.map(|res1|{
            println!("{:?}", res1);
        })
        .map_err(|err| {
            println!("stdout error: {}", err);
        })
        }
         
} */

fn main() {

  //tokio::run(fetch());

    // Notes for reading config file
  let contents = fs::read_to_string("config.json")
    .expect("Something went wrong reading the file");

  let config: Config =
    serde_json::from_str(&contents).expect("JSON was not well-formatted");

  println!("{:?}", config);

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
          for api in job.apis.to_owned() {
            let response = get(&api.url);
            match response {
              Ok(data) => {
                let job_result = config::JobResult{ api_data: data, time: Utc::now() };
                job.sender.send(job_result);
              },
              Err(e) => { eprintln!("Response failed for: '{:?}', error: '{}'", api, e); }
            }
          }
            thread::sleep(Duration::from_secs(job.interval))
        }
      })
    );
  }

  // Continuously listen for data from transitters/senders
  for received in rx {
    println!("Got: {:?}", received);
  }  

/*   // Connect to MongoDB
  let client = mongodb::Client::connect("localhost", 27017)
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
  //println!("{:#?}", resp);
  Ok(resp)
}