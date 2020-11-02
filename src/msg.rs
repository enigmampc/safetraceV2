use crate::bucket::GeoLocationTime;
use crate::data::{ghash, KeyVal};
use cosmwasm_std::{HumanAddr, StdError, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub start_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    AddDataPoints { data_points: Vec<GeoLocationTime> },
    ImportGoogleLocations { data: GoogleTakeoutHistory },
    NewDay {},
    AddAdmin { address: HumanAddr },
    RemoveAdmin { address: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    MatchDataPoint {
        data_point: Vec<GeoLocationTime>,
    },
    HotSpot {
        accuracy: Option<u32>,
        zones: Option<u32>,
    },
    TimeRange {},
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    OverLap { data_ponts: Vec<GeoLocationTime> },
    HotSpotResponse { hot_spots: Vec<HotSpot> },
    DateRange { from: u64, to: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HotSpot {
    pub geo_location: String,
    pub power: u32,
}

impl From<KeyVal> for HotSpot {
    fn from(that: KeyVal) -> Self {
        Self {
            geo_location: that.0,
            power: that.1,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GoogleTakeoutHistory {
    pub locations: Vec<GoogleLocation>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GoogleLocation {
    pub timestampMs: Uint128,
    pub latitudeE7: u64,
    pub longitudeE7: u64,
}

impl TryInto<GeoLocationTime> for GoogleLocation {
    type Error = StdError;

    fn try_into(self) -> StdResult<GeoLocationTime> {
        let geohash = self.hash().map_err(|_| {
            StdError::generic_err(format!(
                "failed to create geohash for ({}, {})",
                self.longitudeE7, self.latitudeE7
            ))
        })?;
        Ok(GeoLocationTime {
            geohash,
            timestamp_ms: self.timestampMs.u128() as u64,
        })
    }
}

impl GoogleLocation {
    pub fn hash(&self) -> StdResult<String> {
        ghash(self.longitudeE7 as f64 / 1e7, self.latitudeE7 as f64 / 1e7)
    }
}
