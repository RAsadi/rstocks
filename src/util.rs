use chrono::offset::Local;
use chrono::{DateTime, TimeZone, Utc};
use slice_deque::SliceDeque;
use std::collections::BTreeMap;

pub struct LoopingIndex {
    pub max_size: usize,
    pub index: usize,
}

impl LoopingIndex {
    pub fn new(max_size: usize) -> LoopingIndex {
        return LoopingIndex {
            max_size: max_size,
            index: 0,
        };
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.max_size;
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.max_size - 1;
        }
    }
}

pub struct SortedBTreeMap {
    btree_map: BTreeMap<String, SliceDeque<(f64, f64)>>,
    max_size: u8,
}

impl SortedBTreeMap {
    pub fn new(max_size: u8) -> SortedBTreeMap {
        return SortedBTreeMap {
            btree_map: BTreeMap::new(),
            max_size: max_size,
        };
    }

    pub fn insert(&mut self, ticker: String, val: (f64, f64)) {
        self.btree_map.entry(ticker.clone()).or_default().push_back(val);
        if self.btree_map[&ticker].len() > self.max_size.into() {
            self.btree_map.entry(ticker.clone()).or_default().pop_front();
        }
    }

    pub fn get_btree_map(&self) -> &BTreeMap<String, SliceDeque<(f64, f64)>> {
        return &self.btree_map;
    }

    pub fn get_min(&self, ticker: String) -> f64 {
        return self.btree_map[&ticker]
            .iter()
            .cloned()
            .fold(1. / 0. /* inf */, |acc, (_, val)| f64::min(acc, val));
    }

    pub fn get_max(&self, ticker: String) -> f64 {
        return self.btree_map[&ticker]
            .iter()
            .cloned()
            .fold(-1. / 0. /* -inf */, |acc, (_, val)| f64::max(acc, val));
    }

    pub fn min_time(&self, ticker: String) -> DateTime<Local> {
        let unix_seconds = self.btree_map[&ticker]
            .iter()
            .cloned()
            .fold(1. / 0. /* inf */, |acc, (time, _)| f64::min(acc, time));
        return Utc.timestamp(unix_seconds as i64, 0).into();
    }

    pub fn max_time(&self, ticker: String) -> DateTime<Local> {
        let unix_seconds = self.btree_map[&ticker]
            .iter()
            .cloned()
            .fold(-1. / 0. /* -inf */, |acc, (time, _)| f64::max(acc, time));
        return Utc.timestamp(unix_seconds as i64, 0).into();
    }
}
