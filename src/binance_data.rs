use std::fs::File;
use std::io::prelude::Read;
use std::io::Cursor;
use std::iter::Iterator;

use yata::core::Candle;

use chrono::prelude::*;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use log::info;
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
    symbol: String,
    interval: String,
    from: NaiveDate,
    to: NaiveDate,
) -> Vec<Candle> {
    let mut cur_date = from;
    let mut result: Vec<Candle> = Vec::new();
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
                let candle = parse_binance_kline(&line).await;
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