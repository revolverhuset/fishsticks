use std::sync::{Arc, Mutex};
use std::time::Duration;

use matrix_bot_api::handlers::{HandleResult, StatelessHandler};
use matrix_bot_api::{MatrixBot, MessageType, BKResponse};

use cmd;
use config;
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
    reminder: Option<&config::MatrixRemainderConfig>,
) -> Result<(), ()> {
    let mut handler = StatelessHandler::new();
    let state_mutex = state.clone();
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

    let mut bot = MatrixBot::new(handler);
    bot.connect(matrix_user, matrix_password, matrix_server);

    let mut connected = false;

    let mut next_reminder = if let Some(reminder) = reminder {
        let now = std::time::SystemTime::now();

        Some(now +
            Duration::from_secs(
                reminder.interval_sec as u64 -
                    now.duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs() as u64
                    % reminder.interval_sec
                    + reminder.offset_sec)
            )
    } else {
        None
    };

    loop {
        let now = std::time::SystemTime::now();

        let cmd = if let Some(this_reminder) = next_reminder {
            let timeout = this_reminder.duration_since(now).unwrap_or(Duration::new(0, 0));
            let cmd = bot.rx.recv_timeout(timeout);

            if let Some(reminder) = reminder {
                if connected && now >= this_reminder {
                    let state = state_mutex.lock().unwrap();

                    if state.demand_open_order().is_ok() {
                        let message = "Use `!ffs sharebill` or `!ffs closeorder` to close the currently open order";
                        bot.send_message(&message, &reminder.channel, MessageType::RoomNotice);
                    }
                    let reminder_interval = Duration::from_secs(reminder.interval_sec);
                    next_reminder = Some(this_reminder + reminder_interval);
                }
            }

            cmd.map_err(|_| ())
        } else {
            bot.rx.recv().map_err(|_| ())
        };

        if let Ok(cmd) = cmd {
            // first handle with matrix_bot_api
            if !bot.handle_recvs(&cmd) {
                break;
            }

            // then custom handlers
            match cmd {
                BKResponse::Token(_, _) => {
                    connected = true;
                }
                _ => (),
            }
        }
    }

    Ok(())
}
