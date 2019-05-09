use std::sync::{Arc, Mutex};

use matrix_bot_api::handlers::{HandleResult, StatelessHandler};
use matrix_bot_api::{MatrixBot, MessageType};

use cmd;
use state;
use web;

pub fn run(
    state: Arc<Mutex<state::State>>,
    env: web::Env,
    matrix_user: &str,
    matrix_password: &str,
    matrix_server: &str,
) -> Result<(), ()> {
    let mut handler = StatelessHandler::new();
    handler.register_handle(
        "ffs",
        Box::new(move |bot, message, tail| {
            println!("{:?}", message);

            let room = &message.room;

            let mut split = tail.splitn(2, ' ');
            let cmd = split.next().unwrap();
            let args = split.next().unwrap_or("");

            let slack_response = cmd::exec_cmd(
                cmd,
                &cmd::CommandContext {
                    state_mutex: &state,
                    args: args,
                    user_name: &message.sender,
                    env: &env,
                },
            )
            .map(cmd::SlackResponse::from);

            match slack_response {
                Ok(slack_response) => {
                    let message_type = match slack_response.response_type {
                        cmd::ResponseType::Ephemeral => MessageType::RoomNotice,
                        cmd::ResponseType::InChannel => MessageType::TextMessage,
                    };

                    bot.send_message(&slack_response.text, room, message_type);
                }
                Err(err) => {
                    bot.send_message(&format!("{:?}", err), room, MessageType::RoomNotice);
                }
            };

            HandleResult::StopHandling
        }),
    );

    let bot = MatrixBot::new(handler);
    bot.run(matrix_user, matrix_password, matrix_server);

    Ok(())
}
