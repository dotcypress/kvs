extern crate kvs;

use kvs::adapters::ram::MemoryAdapter;
use kvs::*;
use std::io::*;

const STORE_SIZE: usize = 128;
const BUCKETS: usize = 8;
const SLOTS: usize = 1;

type CrudStore = KVStore<MemoryAdapter<STORE_SIZE>, BUCKETS, SLOTS>;

#[cfg(feature = "crud")]
#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
enum SensorMode {
    LowPower,
    Normal,
}

#[cfg(feature = "crud")]
#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
struct SensorCalibration {
    offset: usize,
    prescaler: u8,
    mode: SensorMode,
}

fn main() {
    #[cfg(feature = "crud")]
    {
        let mut store =
            CrudStore::open(MemoryAdapter::default(), StoreConfig::new(0xf00d, 8), true).unwrap();
        store
            .create::<SensorCalibration, 32>(
                b"sensor/1",
                &SensorCalibration {
                    offset: 100500,
                    prescaler: 255,
                    mode: SensorMode::LowPower,
                },
            )
            .unwrap();
        store
            .create::<SensorCalibration, 32>(
                b"sensor/2",
                &SensorCalibration {
                    offset: 100,
                    prescaler: 128,
                    mode: SensorMode::Normal,
                },
            )
            .unwrap();
        store
            .create::<SensorCalibration, 32>(
                b"sensor/3",
                &SensorCalibration {
                    offset: 500,
                    prescaler: 1,
                    mode: SensorMode::LowPower,
                },
            )
            .unwrap();
        stdout().write_all(&store.close().memory).ok();
    }
}
