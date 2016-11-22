extern crate iron;
extern crate serde;
extern crate serde_json;
extern crate urlencoded;

use web;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::headers::ContentType;
use self::iron::modifiers::Header;
use self::urlencoded::UrlEncodedBody;

enum ResponseType {
    Ephemeral,
    InChannel,
}

impl serde::Serialize for ResponseType {
    fn serialize<S: serde::Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_str(match *self {
            ResponseType::Ephemeral => "ephemeral",
            ResponseType::InChannel => "in_channel",
        })
    }
}

#[derive(Serialize)]
struct SlackResponse<'a> {
    response_type: ResponseType,
    text: &'a str,
}

pub fn slack(req: &mut Request) -> IronResult<Response> {
    let hashmap = req.get::<UrlEncodedBody>().unwrap();

    println!("Parsed GET request query string:\n {:?}", hashmap);

    if hashmap.contains_key("sslcheck") {
        return Ok(Response::with(status::Ok));
    }

    let _state = req.extensions.get::<web::StateContainer>().unwrap().0.lock().unwrap();

    Ok(Response::with((
        status::Ok,
        serde_json::to_string(&SlackResponse {
            response_type: ResponseType::Ephemeral,
            text: &format!("You said {:?}", &hashmap.get("text").unwrap()),
        }).unwrap(),
        Header(ContentType::json()),
    )))
}
