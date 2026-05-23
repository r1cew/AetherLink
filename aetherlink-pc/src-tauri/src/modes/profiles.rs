use crate::modes::automation;
use aetherlink_common::protocol::ServerResponse;

pub fn run_profile(data_dir: &std::path::PathBuf, profile_id: &str) -> ServerResponse {
    automation::run(data_dir, profile_id)
}
