use std::fs::File;
use std::io::prelude::Read;
use std::io::Cursor;
use std::iter::Iterator;

use yata::core::Candle;
use yata::core::IndicatorResult;
use yata::indicators::MACD;
use yata::prelude::*;

use chrono::prelude::*;
use chrono::{DateTime, NaiveDateTime, Utc};

use log::info;
use reqwest;
use tempfile::tempfile;

fn is_current_month(year: i32, month: u32) -> bool {
    let now = Utc::now();
    let current_year = now.year();
    let current_month = now.month();
    year == current_year && month == current_month
}

fn binance_data_url(symbol: String, interval: String, year: i32, month: u32) -> String {
    let folder = if is_current_month(year, month) {
        "daily"
    } else {
        "monthly"
    };
    let base_url = format!("https://data.binance.vision/data/spot/{}/klines", folder);
    let file_name = format!("{}-{}-{}-{:02}.zip", symbol, interval, year, month);
    let url = format!("{}/{}/{}/{}", base_url, symbol, interval, file_name);
    url
}

async fn check_url_exists(url: String) -> bool {
    let response = reqwest::get(url).await.unwrap();
    response.status().is_success()
}

async fn download_binance_data_to_file(
    url: String,
    target: &mut File,
) -> std::result::Result<(), std::io::Error> {
    let response = reqwest::get(url).await.unwrap();
    let mut content = Cursor::new(response.bytes().await.unwrap());
    std::io::copy(&mut content, target)?;
    Ok(())
}

fn read_zip_file(source: File) -> String {
    let mut archive = zip::ZipArchive::new(source).unwrap();
    let mut data = archive.by_index(0).unwrap();
    let mut buf = String::new();
    data.read_to_string(&mut buf);
    buf
}

async fn parse_binance_kline(data: &str) -> Option<Candle> {
    if !data.contains(",") {
        return None;
    }
    let mut data = data.split(",");
    let start_time: i64 = data.next().unwrap().parse().unwrap();
    let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(start_time / 1000, 0), Utc);
    info!("parsing data at {}", dt);
    let open: f64 = data.next().unwrap().parse().unwrap();
    let close: f64 = data.next().unwrap().parse().unwrap();
    let high: f64 = data.next().unwrap().parse().unwrap();
    let low: f64 = data.next().unwrap().parse().unwrap();
    let volume: f64 = data.next().unwrap().parse().unwrap();
    let parsed = Candle {
        open,
        close,
        high,
        low,
        volume,
    };
    Some(parsed)
}

fn add_one_month(year: i32, month: u32) -> (i32, u32) {
    let result = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    result
}

struct MonthYear {
    year: i32,
    month: u32,
}

async fn get_kline_data(
    symbol: String,
    interval: String,
    from: NaiveDate,
    to: NaiveDate,
) -> Vec<Candle> {
    let mut cur_year = from.year();
    let mut cur_month = from.month();

    while NaiveDate::from_ymd(cur_year, cur_month, 1) < to {
        let url = binance_data_url(
            symbol.to_string(),
            interval.to_string(),
            cur_year,
            cur_month,
        );
        let check = check_url_exists(url.to_string()).await;
        println!("checking url {}:{}", url, check);

        let next_month = add_one_month(cur_year, cur_month);
        cur_year = next_month.0;
        cur_month = next_month.1;
    }
    Vec::new()
}

// ---- backtest ----
// fn available_fund
// fn current_position

// --- end result ---
// fn kline
// fn indicators
// fn signals
// fn profit_and_loss

async fn test_fetch(symbol: &str, interval: &str, year: i32, month: i32) {
    let base_url = "https://data.binance.vision/data/spot/monthly/klines";
    let file_name = format!("{}-{}-{}-{:02}.zip", symbol, interval, year, month);
    let url = format!("{}/{}/{}/{}", base_url, symbol, interval, file_name);
    let response = reqwest::get(url).await.unwrap();
    println!("{}", response.status().is_success());
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
async fn fetch_binance_kline_data(
    symbol: &str,
    interval: &str,
    year: i32,
    month: i32,
    target: &mut File,
) -> Result<()> {
    let base_url = "https://data.binance.vision/data/spot/monthly/klines";
    let file_name = format!("{}-{}-{}-{:02}.zip", symbol, interval, year, month);
    let url = format!("{}/{}/{}/{}", base_url, symbol, interval, file_name);
    let response = reqwest::get(url).await?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, target)?;
    Ok(())
}

fn unpack_binance_kline_data(file: File) -> String {
    let mut archive = zip::ZipArchive::new(file).unwrap();
    let mut file = archive.by_index(0).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    return contents;
}

fn transform_kline_data(data: String) -> Vec<Candle> {
    let lines = data.split("\n");
    let mut result: Vec<Candle> = Vec::new();
    for line in lines {
        if !line.contains(",") {
            continue;
        }
        let mut data = line.split(",");
        let start_time: i64 = data.next().unwrap().parse().unwrap();
        let dt =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(start_time / 1000, 0), Utc);
        info!("loading {}", dt);

        let open: f64 = data.next().unwrap().parse().unwrap();
        let close: f64 = data.next().unwrap().parse().unwrap();
        let high: f64 = data.next().unwrap().parse().unwrap();
        let low: f64 = data.next().unwrap().parse().unwrap();
        let volume: f64 = data.next().unwrap().parse().unwrap();
        let parsed = Candle {
            open,
            close,
            high,
            low,
            volume,
        };
        result.push(parsed);
    }
    return result;
}

#[tokio::main]
pub async fn main() {
    get_kline_data(
        String::from("ETHUSDT"),
        String::from("1d"),
        NaiveDate::from_ymd(2017, 1, 1),
        NaiveDate::from_ymd(2021, 11, 21),
    ).await;
    println!("begin process");
    let mut temp_file = tempfile().unwrap();
    fetch_binance_kline_data("ETHUSDT", "1d", 2021, 7, &mut temp_file)
        .await
        .unwrap();
    println!("downloaded data");
    let raw_data = unpack_binance_kline_data(temp_file);
    let mut data = transform_kline_data(raw_data).into_iter();
    println!("transformed into [{}] kline", data.len());

    let macd = MACD::default();
    let mut macd = macd.init(&data.next().unwrap()).unwrap();

    let mut result: Vec<IndicatorResult> = Vec::new();
    for candle in data {
        result.push(macd.next(&candle));
    }
    println!("calculated into [{}] indicator", result.len());
    println!("result[0] {:#?}", result.get(0).unwrap());
}
