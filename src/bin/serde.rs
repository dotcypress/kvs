#[cfg(feature = "serde")]
mod serde {
    use kvs::adapters::ram::MemoryAdapter;
    use kvs::*;
    use std::io::*;

    const STORE_SIZE: usize = 128;
    const BUCKETS: usize = 8;
    const SLOTS: usize = 1;
    type CrudStore = KVStore<MemoryAdapter<STORE_SIZE>, BUCKETS, SLOTS>;

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    enum SensorMode {
        LowPower,
        Normal,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq)]
    struct SensorConfig {
        offset: usize,
        prescaler: u8,
        mode: SensorMode,
    }

    pub fn dump() {
        let mut store =
            CrudStore::open(MemoryAdapter::default(), StoreConfig::new(0xf00d, 8), true).unwrap();
        store
            .insert_object::<SensorConfig, 32>(
                b"sensor/1",
                &SensorConfig {
                    offset: 100500,
                    prescaler: 255,
                    mode: SensorMode::LowPower,
                },
            )
            .unwrap();
        store
            .insert_object::<SensorConfig, 32>(
                b"sensor/2",
                &SensorConfig {
                    offset: 100,
                    prescaler: 128,
                    mode: SensorMode::Normal,
                },
            )
            .unwrap();
        store
            .insert_object::<SensorConfig, 32>(
                b"sensor/3",
                &SensorConfig {
                    offset: 500,
                    prescaler: 1,
                    mode: SensorMode::LowPower,
                },
            )
            .unwrap();
        stdout().write_all(&store.close().memory).ok();
    }
}

fn main() {
    #[cfg(feature = "serde")]
    {
        serde::dump();
    }
}
