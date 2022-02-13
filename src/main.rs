mod average;
// retrieve 6 months worth of data

use serde::{Serialize, Deserialize};
use rouille::{router, Request, Response};
use yahoo_finance_api as yahoo;
use std::time::{Duration, UNIX_EPOCH};
use csv;

use std::error::Error;
use std::collections::HashMap;
use chrono::prelude::*;
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
                    print!("Value {}", item.close);
                }
                let mut sma = average::simple_moving_average(&stock_data,days);
                 let converted_sma = convert_date(sma);
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
                 let converted_ema = convert_date(ema);
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
                 let converted_macd = convert_date(macd);
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
                Response::json(&rsi)
            },
            _ => Response::empty_404()
        )
     });

    // let mut value_high : Vec<f64> = Vec::new();
    // // print out some high numbers!
    let provider = yahoo::YahooConnector::new();
    let resp = provider.get_latest_quotes("AAPL", "1m").unwrap();
    let quote = resp.quotes().unwrap();
    let mut stock_data = Vec::new();
    for item in quote.iter()
    {
                   stock_data.push(item.clone());
    }
    let mut dupa = average::naive_bayes(stock_data, 8, 0.75);
    for record in dupa.iter(){
        println!("Value and the predicton {:.2} on {}.", record.0, record.1);

    }
}

pub fn read_csv_data() -> Result<Vec<StockCompany>, Box<dyn Error>> {
    let mut stock_comapnies: Vec<StockCompany> = Vec::new();
    let mut reader = csv::Reader::from_path("src/stockCompanies.csv");
    let mut reader = match reader {
        Ok(reader) => reader,
        Err(error) => panic!("Problem opening the file: {:?}", error)
    };
    for result in reader.deserialize() {
        let mut record: StockCompany = result?;
        stock_comapnies.push(record);
    }
    Ok(stock_comapnies)
}

pub fn convert_date(mut indicator: HashMap<&u64, f64>) -> Vec<StockIndicator> {
    let mut company_indicator = Vec::new();
    for record in indicator {
        let time: DateTime<Utc> =
            DateTime::from(UNIX_EPOCH + Duration::from_secs(record.0.clone()));
        company_indicator.push(StockIndicator { date: time.format("%Y-%m-%d").to_string(), value: record.1.clone() });
    };
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
    value : f64
}