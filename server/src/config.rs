use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

use mongodb::Client;
use serde::Serialize;

use crate::adf::Strategy;

pub(crate) const COOKIE_DURATION: actix_web::cookie::time::Duration =
    actix_web::cookie::time::Duration::minutes(30);
pub(crate) const COMPUTE_TIME: Duration = Duration::from_secs(120);

pub(crate) const ASSET_DIRECTORY: &str = "./assets";

pub(crate) const DB_NAME: &str = "adf-obdd";
pub(crate) const USER_COLL: &str = "users";
pub(crate) const ADF_COLL: &str = "adf-problems";

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize)]
pub(crate) enum Task {
    Parse,
    Solve(Strategy),
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct RunningInfo {
    pub(crate) username: String,
    pub(crate) adf_name: String,
    pub(crate) task: Task,
}

pub(crate) struct AppState {
    pub(crate) mongodb_client: Client,
    pub(crate) currently_running: Mutex<HashSet<RunningInfo>>,
}
