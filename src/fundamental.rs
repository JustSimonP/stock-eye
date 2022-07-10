
use std::{fs, env};
use std::convert::TryFrom;
use self::reqwest::blocking;

extern crate reqwest;
//https://financialmodelingprep.com/api/v3/income-statement/AAPL?datatype=csv&apikey=YOUR_API_KEY
//https://financialmodelingprep.com/api/v3/income-statement/FHZN.SW?limit=100&apikey=YOUR_API_KEY


pub fn getCashFlow()  {
    //let mut headers = HeaderMap::new();
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    let API_KEY: String =fs::read_to_string("api_key.txt").expect("Something went wrong reading the file");
        println!("KALABUNGA: {}", API_KEY);
        //headers.insert(UPGRADE_INSECURE_REQUESTS, "1".parse().unwrap());
        let mut url: String = "https://financialmodelingprep.com/api/v3/financial-statement-symbol-lists?apikey=".to_string();
        url.push_str(&API_KEY);
        println!("KALABUNGA: {}", url);
        let res = reqwest::blocking::Client::new()
            .get(&url)
            //.headers(headers)
            .send()
            .unwrap();
        let dupa = res.text().unwrap();


        println!("{}", dupa);


    }
    //Error("API_KEY retrieval was unsuccessful")
