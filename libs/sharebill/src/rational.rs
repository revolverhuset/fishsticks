extern crate num;
extern crate regex;

use std::fmt;
use std::ops;
use std::str::FromStr;

use self::num::{BigRational, One, ToPrimitive, Zero};
use self::regex::Regex;

quick_error! {
    #[derive(Debug)]
    pub enum ParseRationalError {
        InvalidRationalNumber
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Rational(pub num::BigRational);

impl Rational {
    pub fn from_cents(cents: i32) -> Rational {
        Rational(num::BigRational::new(cents.into(), 100.into()))
    }

    pub fn to_f64(&self) -> f64 {
        self.0.numer().to_f64().unwrap() / self.0.denom().to_f64().unwrap()
    }
}

lazy_static! {
    static ref MIXED_NUMBER: Regex = {
        Regex::new(r"^((-)?(\d+)( (\d+/\d+))?|(-?\d+/\d+))$").expect("Error in hard-coded regex")
    };
}

impl FromStr for Rational {
    type Err = ParseRationalError;

    fn from_str(number: &str) -> Result<Rational, ParseRationalError> {
        use self::num::BigRational as BR;

        match MIXED_NUMBER.captures(number) {
            Some(groups) => {
                // The parsing below must succeed because of the regex match, unwrap is ok.
                let mut result = BR::zero();
                if let Some(x) = groups.at(3) {
                    result = result + x.parse::<BR>().unwrap();
                }
                if let Some(x) = groups.at(5) {
                    result = result + x.parse::<BR>().unwrap();
                }
                if let Some(x) = groups.at(6) {
                    result = result + x.parse::<BR>().unwrap();
                }
                if let Some(_) = groups.at(2) {
                    result = -result;
                }

                Ok(Rational(result))
            }
            None => Err(ParseRationalError::InvalidRationalNumber),
        }
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let x = &self.0;

        if x.denom() == &num::BigInt::one() {
            return write!(f, "{}", &x.numer());
        }

        let whole = x.to_integer();

        if whole.is_zero() {
            write!(f, "{}/{}", &x.numer(), &x.denom())
        } else {
            let numer = x.numer() - x.denom() * &whole;
            let denom = x.denom();

            write!(f, "{} {}/{}", &whole, &numer, &denom)
        }
    }
}

use serde;
impl serde::Serialize for Rational {
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        ser.serialize_str(&format!("{}", self))
    }
}

struct RationalVisitor;
impl serde::de::Visitor for RationalVisitor {
    type Value = Rational;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(
            "a string for a rational number, matching ^((-)?(\\d+)( (\\d+/\\d+))?|(-?\\d+/\\d+))$",
        )
    }

    fn visit_str<E>(self, value: &str) -> Result<Rational, E>
    where
        E: serde::de::Error,
    {
        Rational::from_str(value).map_err(|_| E::custom("Nope!"))
    }
}

impl serde::Deserialize for Rational {
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer,
    {
        de.deserialize_str(RationalVisitor)
    }
}

impl<T> From<T> for Rational
where
    num::BigInt: From<T>,
{
    fn from(src: T) -> Rational {
        Rational(BigRational::new(src.into(), num::BigInt::one()))
    }
}

impl Zero for Rational {
    fn zero() -> Rational {
        Rational(BigRational::zero())
    }

    fn is_zero(&self) -> bool {
        self.0.is_zero()
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

impl<'a, 'b> ops::Sub<&'a Rational> for &'b Rational {
    type Output = Rational;
    fn sub(self, other: &Rational) -> Rational {
        Rational(&self.0 - &other.0)
    }
}

impl ops::Div<Rational> for Rational {
    type Output = Rational;
    fn div(self, other: Rational) -> Rational {
        Rational(self.0 / other.0)
    }
}

#[cfg(test)]
mod test {
    use super::num::Zero;
    use super::*;

    #[test]
    fn parse_mixed_number() {
        let actual = "1 1/2".parse::<Rational>().unwrap().0;
        let expected = super::num::BigRational::new(3.into(), 2.into());
        assert_eq!(expected, actual);
    }

    #[test]
    fn format_simple_rational() {
        let r = "1/2".parse::<Rational>().unwrap();
        let actual = format!("{}", &r);
        assert_eq!("1/2", actual);
    }

    #[test]
    fn format_mixed_number() {
        let r = "1 1/2".parse::<Rational>().unwrap();
        let actual = format!("{}", &r);
        assert_eq!("1 1/2", actual);
    }

    #[test]
    fn format_zero() {
        let r = Rational::zero();
        let actual = format!("{}", &r);
        assert_eq!("0", actual);
    }

    #[test]
    fn format_whole_number() {
        let r = "5".parse::<Rational>().unwrap();
        let actual = format!("{}", &r);
        assert_eq!("5", actual);
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
