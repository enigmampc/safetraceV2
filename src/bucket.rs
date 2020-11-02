use std::collections::{BTreeMap, HashMap};
use std::ops::Bound::Included;
use std::slice::Iter;

use bincode2;
use cosmwasm_std::{ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use self::BucketName::*;
use crate::data::{ghash, HotSpots, KeyVal};
use crate::msg::HotSpot;

pub static ONE_DAY: u64 = 1000 * 60 * 60 * 24;
pub static POINTERS_KEY: &[u8] = b"pointers";
pub static BUCKETS_KEY: &[u8] = b"buckets";
//pub static RESERVED: usize = 10;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Hash, Eq)]
pub enum BucketName {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Twelve,
    Thirteen,
    Fourteen,
}

impl BucketName {
    pub fn iterator() -> Iter<'static, BucketName> {
        static DIRECTIONS: [BucketName; 14] = [
            One, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Eleven, Twelve, Thirteen,
            Fourteen,
        ];
        DIRECTIONS.iter()
    }
}

impl Into<&[u8]> for BucketName {
    fn into(self) -> &'static [u8] {
        match self {
            One => b"One",
            Two => b"Two",
            Three => b"Three",
            Four => b"Four",
            Five => b"Five",
            Six => b"Six",
            Seven => b"Seven",
            Eight => b"Eight",
            Nine => b"Nine",
            Ten => b"Ten",
            Eleven => b"Eleven",
            Twelve => b"Twelve",
            Thirteen => b"Thirteen",
            Fourteen => b"Fourteen",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Locations(pub Vec<GeoLocationTime>);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Times(pub Vec<u64>);

impl Default for Times {
    fn default() -> Self {
        let this: Vec<u64> = vec![];
        //this.reserve(RESERVED);

        return Self { 0: this };
    }
}

impl Default for Locations {
    fn default() -> Self {
        let this: Vec<GeoLocationTime> = vec![];
        //this.reserve(RESERVED);

        return Self { 0: this };
    }
}

// Structs
// #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, JsonSchema)]
// pub struct GeoLocationTime {
//     pub lat: f64,
//     pub lng: f64,
//     pub timestamp_ms: u64,
// }
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GeoLocationTime {
    pub geohash: String,
    pub timestamp_ms: u64,
}

impl GeoLocationTime {
    pub fn is_valid(&self) -> bool {
        true
    }
    // pub fn hash(&self) -> StdResult<String> {
    //     ghash(self.lng, self.lat)
    // }
}

// Structs
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Pointer {
    pub start_time: u64,
    pub end_time: u64,
    pub bucket: BucketName,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Bucket {
    //pub locations: BTreeMap<u64, Locations>,
    pub locations: HashMap<String, Times>,
    pub hotzones: Vec<KeyVal>,
}

impl Bucket {
    pub fn store<S: Storage>(&self, store: &mut S, id: &BucketName) -> StdResult<()> {
        let mut config_store = PrefixedStorage::new(BUCKETS_KEY, store);
        let as_bytes = bincode2::serialize(&self)
            .map_err(|_| StdError::generic_err("Error packing pointers"))?;

        config_store.set((*id).into(), &as_bytes);

        Ok(())
    }

    pub fn load<S: Storage>(store: &S, id: &BucketName) -> StdResult<Self> {
        let config_store = ReadonlyPrefixedStorage::new(BUCKETS_KEY, store);
        if let Some(bucket) = config_store.get((*id).into()) {
            let ptrs: Self = bincode2::deserialize(&bucket)
                .map_err(|_| StdError::generic_err("Error deserializing bucket"))?;
            return Ok(ptrs);
        }

        Ok(Self {
            locations: Default::default(),
            hotzones: Default::default(),
        })
    }

    pub fn insert_data_point(&mut self, geotime: GeoLocationTime) {
        let entry = self.locations.entry(geotime.geohash.clone()).or_default();
        entry.0.push(geotime.timestamp_ms);

        if entry.0.len() as u32 > self.hotzones.last().unwrap().1 {
            let _ = self.hotzones.pop();
            self.hotzones
                .push(KeyVal(geotime.geohash, entry.0.len() as u32));
            self.hotzones.sort_unstable_by(|a, b| b.cmp(a));
        }

        //if v > &commons.last().unwrap().1 {
        //             commons.pop();
        //             commons.push(KeyVal(k.clone(), v.clone()));
        //             commons.sort_unstable_by(|a, b| b.cmp(a));
        //         }
    }

    pub fn match_pos(&self, ghash: &String, time: u64, period: u64) -> bool {
        // let mut in_range: Vec<GeoLocationTime> = Vec::default();
        // for (_, v) in (&self)
        //     .locations
        //     .range((Included(&start_time), Included(&(start_time + period))))
        // {
        //     in_range.append(&mut v.0.clone());
        // }
        //
        // in_range
        if let Some(times) = self.locations.get(ghash) {
            for t in &times.0 {
                if &time > t && time < t + period {
                    return true;
                }
            }
        }
        return false;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Pointers(pub Vec<Pointer>);

impl Pointers {
    pub fn store<S: Storage>(&self, store: &mut S) -> StdResult<()> {
        let mut config_store = PrefixedStorage::new(POINTERS_KEY, store);
        let as_bytes = bincode2::serialize(&self)
            .map_err(|_| StdError::generic_err("Error serializing pointers"))?;

        config_store.set(POINTERS_KEY, &as_bytes);

        Ok(())
    }

    pub fn load<S: Storage>(store: &S) -> StdResult<Self> {
        let config_store = ReadonlyPrefixedStorage::new(POINTERS_KEY, store);
        if let Some(temp) = config_store.get(POINTERS_KEY) {
            let ptrs: Self = bincode2::deserialize(&temp)
                .map_err(|_| StdError::generic_err("Error deserializing pointers"))?;
            return Ok(ptrs);
        }

        Ok(Self::default())
    }

    pub fn find_bucket(&self, time: u64) -> Option<BucketName> {
        for p in &self.0 {
            if time >= p.start_time && time <= p.end_time {
                return Some(p.bucket);
            }
        }
        None
    }

    pub fn sort(&mut self) {
        self.0
            .sort_unstable_by(|a, b| a.start_time.cmp(&b.start_time))
    }

    pub fn pop(&mut self) -> Option<Pointer> {
        self.0.pop()
    }

    pub fn insert(&mut self, ptr: Pointer) {
        self.0.push(ptr);
        self.sort();
    }

    pub fn first(&self) -> Option<&Pointer> {
        self.0.first()
    }

    pub fn last(&self) -> Option<&Pointer> {
        self.0.last()
    }
}

pub fn load_all_buckets<S: Storage>(store: &S) -> StdResult<HashMap<BucketName, Bucket>> {
    let mut map = HashMap::<BucketName, Bucket>::default();
    for name in BucketName::iterator() {
        map.insert(name.clone(), Bucket::load(store, name)?);
    }

    Ok(map)
}

pub fn initialize_buckets<S: Storage>(store: &mut S, start_time: u64) -> StdResult<()> {
    let mut cur_time = start_time;
    let mut pointers = Pointers::default();
    for name in BucketName::iterator() {
        let new_pointer = Pointer {
            start_time: cur_time,
            end_time: cur_time + ONE_DAY,
            bucket: name.clone(),
        };
        cur_time = cur_time + ONE_DAY + 1;

        pointers.insert(new_pointer);
    }
    pointers.store(store)
}
