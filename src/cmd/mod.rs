mod command_context;
mod commands;
mod error;
mod response;

pub use self::command_context::CommandContext;
use self::commands::COMMAND_MAP;
pub use self::error::Error;
pub use self::response::*;

pub fn exec_cmd(cmd: &str, cmd_ctx: &CommandContext) -> Result<Response, Error> {
    match COMMAND_MAP.get(cmd) {
        Some(cmd) => cmd(cmd_ctx),
        _ => Ok(Response::UnknownCommand {
            cmd: cmd.to_string(),
            args: cmd_ctx.args.to_string(),
        }),
    }
}
