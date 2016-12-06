extern crate num;
extern crate regex;

use std::fmt;
use std::ops;
use std::str::FromStr;

use self::num::Zero;
use self::regex::Regex;

quick_error! {
    #[derive(Debug)]
    pub enum ParseRationalError {
        InvalidRationalNumber
    }
}

#[derive(Debug, Eq)]
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
                let mut result = BR::zero();
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

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = &self.0;
        let whole = x.to_integer();
        let numer = x.numer() - x.denom() * &whole;
        let denom = x.denom();

        if whole == num::BigInt::zero() {
            write!(f, "{}/{}", &numer, &denom)
        } else {
            write!(f, "{} {}/{}", &whole, &numer, &denom)
        }
    }
}

impl PartialEq for Rational {
    fn eq(&self, other: &Rational) -> bool {
       self.0 == other.0
    }
}

impl ops::Add<Rational> for Rational {
    type Output = Rational;
    fn add(self, other: Rational) -> Rational {
        Rational(self.0 + other.0)
    }
}

impl<'a> ops::Add<Rational> for &'a Rational {
    type Output = Rational;
    fn add(self, other: Rational) -> Rational {
        Rational(&self.0 + other.0)
    }
}

impl<'a> ops::Add<&'a Rational> for Rational {
    type Output = Rational;
    fn add(self, other: &Rational) -> Rational {
        Rational(self.0 + &other.0)
    }
}

impl<'a, 'b> ops::Add<&'a Rational> for &'b Rational {
    type Output = Rational;
    fn add(self, other: &Rational) -> Rational {
        Rational(&self.0 + &other.0)
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

    #[test]
    fn it_formats_simple_rational() {
        let r = "1/2".parse::<Rational>().unwrap();
        let actual = format!("{}", &r);
        assert_eq!("1/2", actual);
    }

    #[test]
    fn it_formats_mixed_number() {
        let r = "1 1/2".parse::<Rational>().unwrap();
        let actual = format!("{}", &r);
        assert_eq!("1 1/2", actual);
    }

    #[test]
    fn eq() {
        let a = "1/2".parse::<Rational>().unwrap();
        let b = "1/2".parse::<Rational>().unwrap();
        assert_eq!(a, b);
    }

    fn fabricate_to_add() -> (Rational, Rational) {
        (
            "1/2".parse::<Rational>().unwrap(),
            "1/3".parse::<Rational>().unwrap(),
        )
    }

    #[test]
    fn add_m_m() {
        let (a, b) = fabricate_to_add();
        assert_eq!("5/6".parse::<Rational>().unwrap(), a + b);
    }

    #[test]
    fn add_m_b() {
        let (a, b) = fabricate_to_add();
        assert_eq!("5/6".parse::<Rational>().unwrap(), a + &b);
    }

    #[test]
    fn add_b_m() {
        let (a, b) = fabricate_to_add();
        assert_eq!("5/6".parse::<Rational>().unwrap(), &a + b);
    }

    #[test]
    fn add_b_b() {
        let (a, b) = fabricate_to_add();
        assert_eq!("5/6".parse::<Rational>().unwrap(), &a + &b);
    }
}