use std::fs::File;
use std::io::prelude::Read;
use std::io::Cursor;
use std::iter::Iterator;

use yata::core::OHLCV;

use chrono::prelude::*;
use chrono::{Duration, NaiveDateTime, Utc};
use reqwest;
use tempfile::tempfile;

fn is_current_month(year: i32, month: u32) -> bool {
    let now = Utc::now();
    let current_year = now.year();
    let current_month = now.month();
    year == current_year && month == current_month
}

fn binance_data_url(symbol: String, interval: String, year: i32, month: u32, day: u32) -> String {
    let folder = if is_current_month(year, month) {
        "daily"
    } else {
        "monthly"
    };
    let base_url = format!("https://data.binance.vision/data/spot/{}/klines", folder);
    let file_name = match folder {
        "daily" => format!(
            "{}-{}-{}-{:02}-{:02}.zip",
            symbol, interval, year, month, day
        ),
        "monthly" => format!("{}-{}-{}-{:02}.zip", symbol, interval, year, month),
        _ => panic!("Not expected folder type"),
    };
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
    data.read_to_string(&mut buf).unwrap();
    buf
}

#[derive(Debug, PartialEq)]
pub struct BinanceKline {
    pub start_time: NaiveDateTime,
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub volume: f64,
    pub end_time: NaiveDateTime,
}

impl OHLCV for BinanceKline {
    fn open(&self) -> f64 {
        self.open
    }
    fn close(&self) -> f64 {
        self.close
    }
    fn high(&self) -> f64 {
        self.high
    }
    fn low(&self) -> f64 {
        self.low
    }
    fn volume(&self) -> f64 {
        self.volume
    }
}

fn parse_binance_kline(data: &str) -> Option<BinanceKline> {
    if !data.contains(",") {
        return None;
    }
    let mut data = data.split(",");
    let start_time: i64 = data.next().unwrap().parse().unwrap();
    let start_time = NaiveDateTime::from_timestamp(start_time / 1000, 0);
    let open: f64 = data.next().unwrap().parse().unwrap();
    let close: f64 = data.next().unwrap().parse().unwrap();
    let high: f64 = data.next().unwrap().parse().unwrap();
    let low: f64 = data.next().unwrap().parse().unwrap();
    let volume: f64 = data.next().unwrap().parse().unwrap();
    let end_time: i64 = data.next().unwrap().parse().unwrap();
    let end_time = NaiveDateTime::from_timestamp(end_time / 1000, 0);

    let parsed = BinanceKline {
        start_time,
        open,
        close,
        high,
        low,
        volume,
        end_time,
    };
    Some(parsed)
}

fn advance_date(current_date: NaiveDate) -> NaiveDate {
    let next_date = if !is_current_month(current_date.year(), current_date.month()) {
        if current_date.month() < 12 {
            NaiveDate::from_ymd(current_date.year(), current_date.month() + 1, 1)
        } else {
            NaiveDate::from_ymd(current_date.year() + 1, 1, 1)
        }
    } else {
        current_date + Duration::days(1)
    };
    next_date
}

pub async fn get_kline_data(
    symbol: &str,
    interval: &str,
    from: NaiveDate,
    to: NaiveDate,
) -> Vec<BinanceKline> {
    let mut cur_date = from;
    let mut result: Vec<BinanceKline> = Vec::new();
    while cur_date < to {
        let url = binance_data_url(
            symbol.to_string(),
            interval.to_string(),
            cur_date.year(),
            cur_date.month(),
            cur_date.day(),
        );
        let check = check_url_exists(url.to_string()).await;
        if check {
            let mut temp_file = tempfile().expect("unable to create temp file");
            download_binance_data_to_file(url, &mut temp_file)
                .await
                .unwrap();
            let content = read_zip_file(temp_file);
            for line in content.split("\n") {
                let candle = parse_binance_kline(&line);
                match candle {
                    Some(data) => result.push(data),
                    None => (),
                }
            }
        }
        cur_date = advance_date(cur_date);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_binance_kline() {
        let test_string: &str = "1635739200000,4191.50000000,4320.00000000,4146.30000000,4302.93000000,88831.99690000,1635753599999,376834938.78850900,216236,45666.95420000,193846769.34658200,0";
        let result = parse_binance_kline(test_string).unwrap();
        let expected = BinanceKline {
            start_time: NaiveDate::from_ymd(2021, 11, 01).and_hms(4, 0, 0),
            open: 4191.5,
            close: 4320.0,
            high: 4146.3,
            low: 4302.93,
            volume: 88831.9969,
            end_time: NaiveDate::from_ymd(2021, 11, 01).and_hms(7, 59, 59),
        };

        assert_eq!(result, expected);
    }
}
