use std::fs::File;
use std::io::prelude::Read;
use std::io::Cursor;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
async fn fetch_binance_kline_data(
    symbol: &str,
    interval: &str,
    year: i32,
    month: i32,
    target: &str,
) -> Result<()> {
    let base_url = "https://data.binance.vision/data/spot/monthly/klines";
    let file_name = format!("{}-{}-{}-{:02}.zip", symbol, interval, year, month);
    let url = format!("{}/{}/{}/{}", base_url, symbol, interval, file_name);
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(target)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

fn unpack_binance_kline_data(file_name: &str) -> String {
    let path = Path::new(file_name);
    let f = File::open(&path).unwrap();
    let mut archive = zip::ZipArchive::new(f).unwrap();
    let mut file = archive.by_index(0).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    return contents;
}

#[derive(Debug, Clone, Copy)]
struct BinanceKline {
    start_time: i64,
    end_time: i64,
    open: f32,
    close: f32,
    high: f32,
    low: f32,
    volume: f32,
}

fn transform_kline_data(data: String) -> Vec<BinanceKline> {
    let lines = data.split("\n");
    let mut result: Vec<BinanceKline> = Vec::new();
    for line in lines {
        if !line.contains(",") {
            continue;
        }
        let mut data = line.split(",");
        let start_time: i64 = data.next().unwrap().parse().unwrap();
        let open: f32 = data.next().unwrap().parse().unwrap();
        let close: f32 = data.next().unwrap().parse().unwrap();
        let high: f32 = data.next().unwrap().parse().unwrap();
        let low: f32 = data.next().unwrap().parse().unwrap();
        let volume: f32 = data.next().unwrap().parse().unwrap();
        let end_time: i64 = data.next().unwrap().parse().unwrap();
        let parsed = BinanceKline {
            start_time,
            end_time,
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
    let temp_file = "data/temp.zip";
    fetch_binance_kline_data("ETHUSDT", "5m", 2021, 10, temp_file)
        .await
        .unwrap();
    let raw_data = unpack_binance_kline_data(temp_file);
    let data = transform_kline_data(raw_data);

    println!("get data count {}", data.len());
}
