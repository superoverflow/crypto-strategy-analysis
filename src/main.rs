mod account;
mod data;
mod indicators;
mod traders;

use account::{Account, Position};
use data::{BinanceKline, get_kline_data};
use chrono::{Duration, NaiveDate, Utc};
use traders::{DCATrader, GenericTrader, HODLTrader, MACDTrader, StakeSize, TradingFee};

use env_logger::Env;
use log::info;

async fn download_kline() -> Vec<BinanceKline> {
    let start_date = NaiveDate::from_ymd(2020, 1, 1);
    let end_date = Utc::today() - Duration::days(1);
    let end_date = end_date.naive_utc();
    let symbol = "BTCUSDT";
    let interval = "1d";
    info!(
        "download data from binance for [{}/{}] from [{}] to [{}]",
        symbol, interval, start_date, end_date
    );
    let klines = get_kline_data(symbol, interval, start_date, end_date).await;
    info!("downloaded [{}] klines", klines.len());
    klines
}

fn initialise_acount(klines: Vec<BinanceKline>) -> Account {
    info!("setting up account");
    let first_kline = klines.first().expect("no klines fetched");
    let start_time = first_kline.start_time;
    let start_fund = 1000.0;
    let start_position = Position {
        quantity: 0.0,
        cost: 0.0,
    };
    let account = Account::new(start_fund, start_position, start_time);
    account
}

#[allow(dead_code)]
fn initialise_macd_trader<'a>(
    klines_iter: &'a mut dyn Iterator<Item = BinanceKline>,
) -> MACDTrader {
    info!("setting up macd trader");
    let stake_size = StakeSize::FixPercentage(1.);
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = MACDTrader::new(klines_iter, trading_fee, stake_size);
    trader
}

#[allow(dead_code)]
fn initialise_hodl_trader<'a>(
    klines_iter: &'a mut dyn Iterator<Item = BinanceKline>,
) -> HODLTrader {
    info!("setting up hodl trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = HODLTrader::new(klines_iter, trading_fee);
    trader
}

#[allow(dead_code)]
fn initialise_dca_trader<'a>(klines_iter: &'a mut dyn Iterator<Item = BinanceKline>) -> DCATrader {
    info!("setting up dca trader");
    let trading_fee = TradingFee::PercentageFee(0.005);
    let trader = DCATrader::new(klines_iter, trading_fee);
    trader
}

fn backtest<'a, T>(trader: &mut T, account: &mut Account)
where
    T: GenericTrader<'a>,
{
    info!("running backtest");
    loop {
        let kline = trader.next_trade_session(account);
        match kline {
            Some(kline) => {
                account.mark_to_market(kline.end_time, kline.close);
            }
            None => break,
        }
    }
}




#[tokio::main]
pub async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let klines = download_kline().await;
    let mut klines_iter = klines.clone().into_iter();
    let mut account = initialise_acount(klines);
    let mut trader = initialise_macd_trader(&mut klines_iter);
    backtest(&mut trader, &mut account);
    info!("{:?}", account.profit_and_loss_history.last().unwrap());
    info!("available fund: {} / position: {:?}", account.available_fund, account.position);
}
