use crate::data::BinanceKline;
use crate::indicators::BinanceIndicatorInstance;
use chrono::{Datelike, NaiveDateTime, NaiveDate};
use yata::core::{Action, Error, IndicatorResult, OHLCV};
use yata::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct DCA {}

#[derive(Debug, Clone, Copy)]
pub struct DCAInstance {
    cfg: DCA,
    last_timestamp: NaiveDateTime,
}

impl IndicatorConfig for DCA {
    type Instance = DCAInstance;
    const NAME: &'static str = "DCA";
    fn init<T: OHLCV>(self, _candle: &T) -> Result<Self::Instance, Error> {
        Ok(Self::Instance {
            last_timestamp: NaiveDate::from_ymd(2000, 1, 1).and_hms(0, 0, 0),  // FIXME: a magic date before crypto happens
            cfg: self,
        })
    }
    fn validate(&self) -> bool {
        true
    }
    fn set(&mut self, _name: &str, _value: String) -> Result<(), Error> {
        Ok(())
    }
    fn size(&self) -> (u8, u8) {
        (0, 1)
    }
}

impl Default for DCA {
    fn default() -> Self {
        Self {}
    }
}

impl IndicatorInstance for DCAInstance {
    type Config = DCA;

    fn config(&self) -> &Self::Config {
        &self.cfg
    }

    fn next<T: OHLCV>(&mut self, _candle: &T) -> IndicatorResult {
        IndicatorResult::new(&[], &[])
    }
}

impl BinanceIndicatorInstance for DCAInstance {
    fn next_binance_kline(&mut self, candle: &BinanceKline) -> IndicatorResult {
        let current_time = (*candle).start_time;
        let current_month = current_time.month();
        let last_month = self.last_timestamp.month();
        let action = if current_month != last_month {
            Action::Buy(1)
        } else {
            Action::None
        };
        self.last_timestamp = current_time;
        IndicatorResult::new(&[], &[action])
    }
}