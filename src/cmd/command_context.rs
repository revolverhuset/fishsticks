use state;
use web;

use std::sync::Mutex;

pub struct CommandContext<'a, 'b, 'c, 'd> {
    pub state_mutex: &'a Mutex<state::State>,
    pub args: &'b str,
    pub user_name: &'c str,
    pub env: &'d web::Env,
}
