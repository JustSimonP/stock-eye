mod average;
mod database;
// retrieve 6 months worth of data

use serde::{Serialize, Deserialize};
use rouille::{router, Request, Response, input};
use yahoo_finance_api as yahoo;
use std::time::{Duration, UNIX_EPOCH};
use csv;

use std::error::Error;
use std::collections::HashMap;
use chrono::prelude::*;
use math::round::ceil;
use yahoo_finance_api::Quote;
use mongodb::{
    bson::{doc, Bson},
    sync::Client,
};
use rouille::try_or_400;
use mongodb::sync::{Database, Collection};
#[derive(Serialize)]
struct StockData {
     date: u64,
     value: f64,
}
fn main() {
    rouille::start_server("localhost:3000", move |request| {
        router!(request,
            (GET) (/stocklist) => {
                let stock_companies = read_csv_data().unwrap();
                Response::json(&stock_companies).with_additional_header("Access-Control-Allow-Origin","*")
            },

            (GET) (/sma/{symbol : String}/{period : String}/{days : i16}) => {
                let provider = yahoo::YahooConnector::new();
                let resp =  provider.get_quote_range(symbol.as_str(),"1d",period.as_str()).unwrap();
                let quote = resp.quotes().unwrap();
                let mut stock_data = Vec::new();
                for item in quote.iter()
                {
                    stock_data.push(item.clone());
                }
                let mut sma = average::simple_moving_average(&stock_data, days);
                let converted_sma = convert_date(sma, &stock_data);
                Response::json(&converted_sma).with_additional_header("Access-Control-Allow-Origin","*")
            },
             (GET) (/ema/{symbol : String}/{period : String}/{days : i16}) => {
                let provider = yahoo::YahooConnector::new();
                let resp =  provider.get_quote_range(symbol.as_str(),"1d",period.as_str()).unwrap();
                let quote = resp.quotes().unwrap();
                let mut stock_data = Vec::new();
                for item in quote.iter()
                {
                    stock_data.push(item.clone());
                }
                let mut ema = average::exponential_moving_average(&stock_data,days);
                 let converted_ema = convert_date(ema, &stock_data);
                   Response::json(&converted_ema).with_additional_header("Access-Control-Allow-Origin","*")
            },
             (GET) (/macd/{symbol : String}/{period : String}) => {
                let provider = yahoo::YahooConnector::new();
                let resp =  provider.get_quote_range(symbol.as_str(),"1d",period.as_str()).unwrap();
                let quote = resp.quotes().unwrap();
                let mut stock_data = Vec::new();
                for item in quote.iter()
                {
                    stock_data.push(item.clone());
                }
                let mut macd = average::macd(&stock_data);
                 let converted_macd = convert_date(macd, &stock_data);
                Response::json(&converted_macd).with_additional_header("Access-Control-Allow-Origin","*")
            },
             (GET) (/rsi/{symbol : String}/{period : String}) => {
                let provider = yahoo::YahooConnector::new();
                let resp =  provider.get_quote_range(symbol.as_str(),"1d",period.as_str()).unwrap();
                let quote = resp.quotes().unwrap();
                let mut stock_data = Vec::new();
                for item in quote.iter()
                {
                    stock_data.push(item.clone());
                }
                let mut rsi = average::rsi(&stock_data, 8);
                 //let converted_rsi = convert_date(rsi);
                Response::json(&rsi).with_additional_header("Access-Control-Allow-Origin","*")
            },
            (GET) (/bayes/{symbol : String}/{period : String}/{ratio : f64}/{days : i16}) => {
                let provider = yahoo::YahooConnector::new();
                let resp =  provider.get_quote_range(symbol.as_str(),"1d",period.as_str()).unwrap();
                let quote = resp.quotes().unwrap();
                let mut stock_data = Vec::new();
                for item in quote.iter()
                {
                    stock_data.push(item.clone());
                }
                let mut bayes = average::naive_bayes(stock_data, days, ratio);
                let mut bayes_results = Vec::new();
                for record in bayes.iter() {
                    bayes_results.push(BayesResult {
                        recorded : record.0 as i16,
                        predicted : record.1 as i16
                    });
                }
                Response::json(&bayes_results).with_additional_header("Access-Control-Allow-Origin","*")
            },
             (POST) (/comment/{symbol : String}/{method : String}) => {
                #[derive(Deserialize, Serialize)]
                struct Comment {
                    date : String,
                    body : String,
                    method : String,
                    symbol : String
                }
                let comment : Comment = try_or_400!(rouille::input::json::json_input(request));
                let db = database::get_database();
                let collection = db.collection::<Comment>("stocks");
                let db_result = collection.insert_one(comment, None);
                Response::text("ALL OK").with_additional_header("Access-Control-Allow-Origin","*")
             },
            (GET) (/comment/{symbol : String}) => {
                #[derive(Deserialize, Serialize)]
                struct Comment {
                    date : String,
                    body : String,
                    method : String,
                    symbol : String
                }
                let db = database::get_database();
                let filter = doc! {"symbol" : symbol};
                let result : Collection<Comment> =  db.collection("stocks");
                let cursor = result.find(filter, None).unwrap();
                let mut documents : Vec<Comment> = Vec::new();
                for doc in cursor {
                   documents.push(doc.unwrap());
                }
                Response::json(&documents)
            },
            _ => Response::empty_404()
        )
    });
}

pub fn read_csv_data() -> Result<Vec<StockCompany>, Box<dyn Error>> {
    let mut stock_comapnies: Vec<StockCompany> = Vec::new();
    let reader = csv::Reader::from_path("src/stockCompanies.csv");
    let mut reader = match reader {
        Ok(reader) => reader,
        Err(error) => panic!("Problem opening the file: {:?}", error)
    };
    for result in reader.deserialize() {
        let record: StockCompany = result?;
        stock_comapnies.push(record);
    }
    Ok(stock_comapnies)
}

pub fn convert_date(mut indicator: HashMap<&u64, f64>, data : &Vec<Quote> ) -> Vec<StockIndicator> {
    let mut company_indicator = Vec::new();
    for data_record in data {
        if indicator.contains_key(&data_record.timestamp) {
            let record = indicator.get(&data_record.timestamp).unwrap();
            let time: DateTime<Utc> =
                DateTime::from(UNIX_EPOCH + Duration::from_secs(data_record.timestamp));
            company_indicator.push(StockIndicator { date: time.format("%Y-%m-%d").to_string(),
                value: ceil( *record,2),
                close: ceil(data_record.close,2)});
        }
    }
    company_indicator.sort_by(|a, b| a.date.cmp(&b.date));
    company_indicator
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StockCompany {
     sector: String,
     name: String,
     symbol: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StockIndicator {
    date: String,
    value : f64,
    close : f64
}

#[derive(Deserialize, Serialize)]
pub struct BayesResult {
    recorded : i16,
    predicted : i16
}

// #[derive(Debug, Deserialize, Serialize)]
// pub struct Comment {
//     date : String,
//     body : String,
//     method : String,
//     symbol : String
// }