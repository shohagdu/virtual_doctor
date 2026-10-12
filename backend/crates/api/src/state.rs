use std::sync::Arc;

use vd_ai::ClaudeClient;
use vd_common::Config;
use vd_db::MySqlPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
    pub claude: ClaudeClient,
}
