extern crate time;

use rational::Rational;
use serde;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct Meta {
    pub description: String,

    #[serde(serialize_with = "serialize_time")]
    pub timestamp: time::Tm,
}

#[derive(Serialize)]
pub struct Transaction {
    #[serde(rename="debets")]
    pub debits: HashMap<String, Rational>,

    pub credits: HashMap<String, Rational>,
}

#[derive(Serialize)]
pub struct Post {
    pub meta: Meta,
    pub transaction: Transaction,
}

fn serialize_time<S>(t: &time::Tm, ser: &mut S) -> Result<(), S::Error>
    where S: serde::Serializer
{
    ser.serialize_str(&format!("{}", t.rfc3339()))
}

#[cfg(test)]
mod test {
    extern crate serde_json;

    use super::*;
    use super::time;
    use std::collections::HashMap;

    fn fabricate_meta() -> Meta {
        Meta {
            description: "Fabricated post".to_owned(),
            timestamp: time::strptime("2016-01-01T12:00:00", "%FT%T").unwrap(),
        }
    }

    fn fabricate_transaction() -> Transaction {
        Transaction {
            debits: HashMap::new(),
            credits: HashMap::new(),
        }
    }

    fn fabricate_post() -> Post {
        Post {
            meta: fabricate_meta(),
            transaction: fabricate_transaction(),
        }
    }

    #[test]
    fn serializes_well() {
        let expected = "{\"meta\":{\"description\":\"Fabricated post\",\
            \"timestamp\":\"2016-01-01T12:00:00Z\"},\
            \"transaction\":{\"debets\":{},\"credits\":{}}}";

        let post = fabricate_post();
        let serialized = serde_json::to_string(&post).unwrap();
        assert_eq!(&expected, &serialized);
    }
}
