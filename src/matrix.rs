use std::sync::{Arc, Mutex};

use matrix_bot_api::handlers::{HandleResult, StatelessHandler};
use matrix_bot_api::{MatrixBot, MessageType};

use cmd;
use slack::{ResponseType, SlackResponse};
use state;
use web;

struct MatrixResponse {
    text: String,
    msg_type: MessageType,
}

impl From<SlackResponse> for MatrixResponse {
    fn from(src: SlackResponse) -> Self {
        let msg_type = match src.response_type {
            ResponseType::Ephemeral => MessageType::RoomNotice,
            ResponseType::InChannel => MessageType::TextMessage,
        };

        Self {
            text: src.text,
            msg_type,
        }
    }
}

impl From<cmd::Response> for MatrixResponse {
    fn from(src: cmd::Response) -> Self {
        use cmd::Response::*;
        match src {
            OpenedOrder {
                menu_url,
                restaurant_name,
            } => MatrixResponse {
                text: format!(
                    "🔔 Now taking orders from the {} menu ({}) 📝",
                    restaurant_name, menu_url
                ),
                msg_type: MessageType::TextMessage,
            },
            Sharebill { url } => MatrixResponse {
                text: format!("💸 Posted to Sharebill and closed order ✔️ {}", url),
                msg_type: MessageType::TextMessage,
            },
            x => SlackResponse::from(x).into(),
        }
    }
}

impl From<cmd::Error> for MatrixResponse {
    fn from(src: cmd::Error) -> Self {
        use cmd::Error::*;
        match src {
            StateError(state::Error::OrderAlreadyOpen(_current_open_order)) => MatrixResponse {
                text: format!("🙅 I already have an open order"),
                msg_type: MessageType::RoomNotice,
            },
            x => MatrixResponse {
                text: format!("{:?}", x),
                msg_type: MessageType::RoomNotice,
            },
        }
    }
}

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

            let response = cmd::exec_cmd(
                cmd,
                &cmd::CommandContext {
                    state_mutex: &state,
                    args: args,
                    user_name: &message.sender,
                    env: &env,
                },
            )
            .map(MatrixResponse::from)
            .unwrap_or_else(MatrixResponse::from);

            bot.send_message(&response.text, room, response.msg_type);

            HandleResult::StopHandling
        }),
    );

    let bot = MatrixBot::new(handler);
    bot.run(matrix_user, matrix_password, matrix_server);

    Ok(())
}
