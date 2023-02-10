// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod behavior;
mod error;

use crate::behavior::{EventQueryType, EventStore, Interval};
use chrono::{DateTime, Utc};
use wasm_bindgen::prelude::*;
use web_sys::{console};
use js_sys::Date;

#[wasm_bindgen(start)]
fn main() {
    println!("Hello, world!");
    console::log_1(&"hi".into());
}

#[wasm_bindgen]
pub struct EventStoreService {
    pub(crate) event_store: EventStore
}

#[wasm_bindgen]
impl EventStoreService {

    #[wasm_bindgen(constructor)]
    pub fn new() -> EventStoreService {
        EventStoreService { event_store: EventStore::new() }
    }

    #[wasm_bindgen(js_name = recordEvent)]
    pub fn record_event(&mut self, event_id: String) {
        match self.event_store.record_event(event_id, Some(Utc::now())) {
            Ok(_v) => { },
            Err(_v) => {
                console::log_1(&"an error occurred".into())
            }
        };
    }

    #[wasm_bindgen(js_name = recordEventForDate)]
    pub fn record_event_at_date(&mut self, event_id: String, date: Date) {
        let d: DateTime<Utc> = DateTime::parse_from_rfc3339(&date.to_iso_string().as_string().unwrap()).map(|result| result.into()).unwrap();
        console::log_1(&d.to_string().into());
        match self.event_store.record_event(event_id, Some(d)) {
            Ok(_v) => { },
            Err(_v) => {
                console::log_1(&"an error occurred".into())
            }
        };
    }

    #[wasm_bindgen(js_name = query)]
    pub fn query(&mut self,
        event_id: String,
        interval: Interval,
        num_buckets: usize,
        starting_bucket: usize,
        query_type: EventQueryType,
    ) -> Result<f64, JsValue> {
        match self.event_store.query(
            event_id, interval, num_buckets, starting_bucket, query_type
        ) {
            Ok(v) => Ok(v),
            Err(e) => {
                console::error_2(&"an error occurred".into(), &e.to_string().into());
                Ok(0.0)
            }
        }
    }

}