extern crate num;
extern crate regex;

use std::str::FromStr;

use self::regex::Regex;

quick_error! {
    #[derive(Debug)]
    pub enum ParseRationalError {
        InvalidRationalNumber
    }
}

#[derive(Debug)]
pub struct Rational(num::BigRational);

lazy_static! {
    static ref MIXED_NUMBER: Regex = {
        // A regex error here is a problem with the regular expression, unwrap is ok.
        Regex::new(r"^((-)?(\d+)( (\d+/\d+))?|(-?\d+/\d+))$").unwrap()
    };
}

impl FromStr for Rational {
    type Err = ParseRationalError;

    fn from_str(number : &str) -> Result<Rational, ParseRationalError> {
        use self::num::BigRational as BR;

        match MIXED_NUMBER.captures(number) {
            Some(groups) => {
                // The parsing below must succeed because of the regex match, unwrap is ok.
                let mut result = "0".parse::<BR>().unwrap();
                if let Some(x) = groups.at(3) { result = result + x.parse::<BR>().unwrap(); }
                if let Some(x) = groups.at(5) { result = result + x.parse::<BR>().unwrap(); }
                if let Some(x) = groups.at(6) { result = result + x.parse::<BR>().unwrap(); }
                if let Some(_) = groups.at(2) { result = -result; }

                Ok(Rational(result))
            },
            None => Err(ParseRationalError::InvalidRationalNumber)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_can_parse_mixed_number() {
        let actual = "1 1/2".parse::<Rational>().unwrap().0;
        let expected = super::num::BigRational::new(3.into(), 2.into());
        assert_eq!(expected, actual);
    }
}
