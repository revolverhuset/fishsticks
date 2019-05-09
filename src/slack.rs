use cmd::{exec_cmd, CommandContext, Error, SlackResponse};
use web;

use iron::headers::ContentType;
use iron::modifiers::Header;
use iron::prelude::*;
use iron::status;
use urlencoded::UrlEncodedBody;

fn slack_core(maybe_slack_token: &Option<&str>, req: &mut Request) -> Result<SlackResponse, Error> {
    let hashmap = req.get::<UrlEncodedBody>()?;

    if let &Some(slack_token) = maybe_slack_token {
        let given_token = hashmap
            .get("token")
            .and_then(|tokens| tokens.get(0))
            .map(String::as_ref);

        if given_token != Some(slack_token) {
            return Err(Error::InvalidSlackToken);
        }
    }

    if hashmap.contains_key("sslcheck") {
        return Ok(SlackResponse {
            text: String::new(),
            ..Default::default()
        });
    }

    let ref state_mutex = req.extensions.get::<web::StateContainer>().unwrap().0;
    let ref env = req.extensions.get::<web::EnvContainer>().unwrap().0;

    let text = &hashmap.get("text").ok_or(Error::MissingArgument("text"))?[0];
    let mut split = text.splitn(2, ' ');
    let cmd = split.next().unwrap();
    let args = split.next().unwrap_or("");

    let user_name = &hashmap
        .get("user_name")
        .ok_or(Error::MissingArgument("user_name"))?[0];

    exec_cmd(
        cmd,
        &CommandContext {
            state_mutex: &state_mutex,
            args: args,
            user_name: user_name,
            env: &env,
        },
    )
    .map(Into::into)
}

pub fn slack(slack_token: &Option<&str>, req: &mut Request) -> IronResult<Response> {
    match slack_core(slack_token, req) {
        Ok(response) => Ok(Response::with((
            status::Ok,
            serde_json::to_string(&response).unwrap(),
            Header(ContentType::json()),
        ))),
        Err(err) => Ok(Response::with((
            status::Ok,
            serde_json::to_string(&SlackResponse {
                text: format!("🙅 {:?}", &err),
                ..Default::default()
            })
            .unwrap(),
            Header(ContentType::json()),
        ))),
    }
}
