mod config;

use std::fs;
use config::Config;
use std::collections::HashMap;
use serde_json::{Value};

use mongodb::{Bson, bson, doc};
use mongodb::{Client, ThreadedClient};
use mongodb::db::ThreadedDatabase;

fn main() {

  /* 
  // Notes for reading config file
  let contents = fs::read_to_string("config.json")
    .expect("Something went wrong reading the file");

  let config: Value =
    serde_json::from_str(&contents).expect("JSON was not well-formatted");

  println!("{:?}", config);

  get(&config.apis[0].url); 
   */

  // Connect to MongoDB
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
  };
}

// Tries to parse given url as generic json value
fn get(url: &String) -> Result<Value, Box<std::error::Error>> {
  let resp: Value = reqwest::get(url)?
    .json()?;
  println!("{:#?}", resp);
  Ok(resp)
}