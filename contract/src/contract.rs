use crate::bucket::initialize_buckets;
use crate::data::{import_location_data, match_data_point};
use crate::hotspotmap::HotSpots;
use crate::msg::{HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use crate::state::{config, config_read, State};
use crate::time::{new_day, query_dates};
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryResult,
    StdError, StdResult, Storage,
};

/// Initialize the contract with the start time, and the contract administrator
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        admin: vec![env.message.sender],
    };

    config(&mut deps.storage).save(&state)?;

    // initialize data buckets
    initialize_buckets(&mut deps.storage, msg.start_time)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    let state = config_read(&deps.storage).load()?;

    if !state.admin.contains(&env.message.sender) {
        return Err(StdError::generic_err(
            "You cannot functions from non-admin address".to_string(),
        ));
    }

    // see msg.rs for more details
    match msg {
        // add an admin
        HandleMsg::AddAdmin { address } => add_admin(deps, env, address),
        // remove an admin
        HandleMsg::RemoveAdmin { address } => remove_admin(deps, env, address),
        // signal that a day has passed
        HandleMsg::ChangeDay {} => new_day(deps, env),
        // import new geolocation data
        HandleMsg::ImportGoogleLocations { data } => import_location_data(deps, env, data),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::MatchDataPoints { data_points } => match_data_point(deps, data_points),
        QueryMsg::HotSpot { accuracy, zones } => hotspots(deps, accuracy, zones),
        QueryMsg::TimeRange {} => query_dates(deps),
    }
}

pub fn hotspots<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    _accuracy: Option<u32>,
    _zones: Option<u32>,
) -> QueryResult {
    let res = HotSpots::load(&deps.storage)?;

    return to_binary(&QueryAnswer::HotSpotResponse { hot_spots: res.0 });
}

pub fn add_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut state = config(&mut deps.storage).load()?;

    if !state.admin.contains(&address) {
        state.admin.push(address);
        config(&mut deps.storage).save(&state)?;
    }

    Ok(HandleResponse::default())
}

pub fn remove_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let mut state = config(&mut deps.storage).load()?;

    if let Some(index) = state.admin.iter().position(|a| a == &address) {
        state.admin.remove(index);
        config(&mut deps.storage).save(&state)?;
    }

    Ok(HandleResponse::default())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::time::{Duration, Instant};

    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{Coin, Env, Extern, HumanAddr, InitResponse, MemoryStorage, StdResult};
    use serde::{Deserialize, Serialize};
    use serde_json;

    use crate::contract::init;
    use crate::data::import_location_data;
    use crate::msg::HandleMsg::ImportGoogleLocations;
    use crate::msg::InitMsg;

    pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

    pub fn init_deps(
        canonical_length: usize,
        contract_balance: &[Coin],
    ) -> Extern<MockStorage, MockApi, MockQuerier> {
        let contract_addr = HumanAddr::from(MOCK_CONTRACT_ADDR);
        Extern {
            storage: MemoryStorage::default(),
            api: MockApi::new(canonical_length),
            querier: MockQuerier::new(&[(&contract_addr, contract_balance)]),
        }
    }

    fn init_helper() -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
        Env,
    ) {
        let mut deps = init_deps(20, &[]);
        let env = mock_env("instantiator", &[]);

        let init_msg = InitMsg {
            start_time: 1600129528955,
        };

        (init(&mut deps, env.clone(), init_msg), deps, env)
    }

    fn load_google_data() -> Vec<u8> {
        let mut cert = vec![];
        let mut f = File::open("tests/data/datamsg2.json").unwrap();
        f.read_to_end(&mut cert).unwrap();

        cert
    }

    #[test]
    pub fn test_add_google_data() {
        let data_msg: crate::msg::HandleMsg = serde_json::from_slice(&load_google_data()).unwrap();

        let (res, mut deps, env) = init_helper();
        let now = Instant::now();
        crate::contract::handle(&mut deps, env, data_msg);
        println!("elapsed: {}", now.elapsed().as_millis());
    }
}
