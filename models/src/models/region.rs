use std::str::FromStr;
use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Region {
    Frankfurt,
    NewYork,
}

pub struct RegionParseError;

impl fmt::Display for RegionParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not parse region")
    }
}

impl fmt::Debug for RegionParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl FromStr for Region {
    type Err = RegionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Frankfurt" => Ok(Region::Frankfurt),
            "NewYork" => Ok(Region::NewYork),
            _ => Err(RegionParseError),
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Region::Frankfurt => write!(f, "Frankfurt"),
            Region::NewYork => write!(f, "NewYork"),
           
        }
    }
}