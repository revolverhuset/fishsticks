use state;
use std;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        StateError(err: state::Error) { from() }
        UrlDecodingError(err: urlencoded::UrlDecodingError) { from() }
        PoisonError
        InputError { from(std::num::ParseFloatError) }
        InvalidSlackToken
        MissingAssociation(slack_name: String)
        SerdeJson(err: serde_json::Error) { from() }
        UnexpectedStatus(status: reqwest::StatusCode)
        NotFound
        MissingConfig(config_path: &'static str)
        FormatError(err: std::fmt::Error) { from() }
        ReqwestError(err: reqwest::Error) { from() }
        MissingArgument(arg: &'static str)
    }
}

impl<T> std::convert::From<std::sync::PoisonError<T>> for Error {
    fn from(_err: std::sync::PoisonError<T>) -> Self {
        Error::PoisonError
    }
}
