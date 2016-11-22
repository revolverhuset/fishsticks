extern crate iron;
extern crate serde_json;
extern crate urlencoded;

use web;

use self::iron::prelude::*;
use self::iron::status;
use self::iron::headers::ContentType;
use self::urlencoded::UrlEncodedBody;

#[derive(Serialize)]
struct SlackResponse<'a> {
    text: &'a str,
}

pub fn log_params(req: &mut Request) -> IronResult<Response> {
    // Extract the decoded data as hashmap, using the UrlEncodedQuery plugin.
    let hashmap = req.get::<UrlEncodedBody>().unwrap();

    println!("Parsed GET request query string:\n {:?}", hashmap);

    let _state = req.extensions.get::<web::StateContainer>().unwrap().0.lock().unwrap();

    let mut res = Response::with((
        status::Ok,
        serde_json::to_string(&SlackResponse {
            text: &format!("You said {:?}", &hashmap.get("text"))
        }).unwrap()
    ));

    res.headers.set(ContentType::json());

    Ok(res)
}
