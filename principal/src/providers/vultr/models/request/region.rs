use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum Region {
	Asia(Asia),
	Australia(Australia),
	Europe(Europe),
	NorthAmerica(NorthAmerica),
	SouthAmerica(SouthAmerica),
	Africa(Africa),
	Unknown,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Asia {
	Tokyo,
	Osaka,
	Seoul,
	Singapore,
	Mumbai,
	TelAviv,
	Bangalore,
	Delhi,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Australia {
	Sydney,
	Melbourne,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Europe {
	Amsterdam,
	London,
	Frankfurt,
	Paris,
	Warsaw,
	Madrid,
	Stockholm,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NorthAmerica {
	NewJersey,
	Chicago,
	Dallas,
	Seattle,
	LosAngeles,
	Atlanta,
	SiliconValley,
	Toronto,
	Miami,
	MexicoCity,
	Honolulu,
}

#[derive(Debug, PartialEq, Clone)]
pub enum SouthAmerica {
	SaoPaulo,
	Santiago,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Africa {
	Johannesburg,
}

impl FromStr for Region {
    type Err = ();  // define type of error

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Tokyo" => Ok(Region::Asia(Asia::Tokyo)),
            "Osaka" => Ok(Region::Asia(Asia::Osaka)),
            "Seoul" => Ok(Region::Asia(Asia::Seoul)),
            "Singapore" => Ok(Region::Asia(Asia::Singapore)),
            "Mumbai" => Ok(Region::Asia(Asia::Mumbai)),
            "Tel Aviv" => Ok(Region::Asia(Asia::TelAviv)),
            "Bangalore" => Ok(Region::Asia(Asia::Bangalore)),
            "Delhi" => Ok(Region::Asia(Asia::Delhi)),

            "Sydney" => Ok(Region::Australia(Australia::Sydney)),
            "Melbourne" => Ok(Region::Australia(Australia::Melbourne)),

            "Amsterdam" => Ok(Region::Europe(Europe::Amsterdam)),
            "London" => Ok(Region::Europe(Europe::London)),
            "Frankfurt" => Ok(Region::Europe(Europe::Frankfurt)),
            "Paris" => Ok(Region::Europe(Europe::Paris)),
            "Warsaw" => Ok(Region::Europe(Europe::Warsaw)),
            "Madrid" => Ok(Region::Europe(Europe::Madrid)),
            "Stockholm" => Ok(Region::Europe(Europe::Stockholm)),

            "New Jersey" => Ok(Region::NorthAmerica(NorthAmerica::NewJersey)),
            "Chicago" => Ok(Region::NorthAmerica(NorthAmerica::Chicago)),
            "Dallas" => Ok(Region::NorthAmerica(NorthAmerica::Dallas)),
            "Seattle" => Ok(Region::NorthAmerica(NorthAmerica::Seattle)),
            "Los Angeles" => Ok(Region::NorthAmerica(NorthAmerica::LosAngeles)),
            "Atlanta" => Ok(Region::NorthAmerica(NorthAmerica::Atlanta)),
            "Silicon Valley" => Ok(Region::NorthAmerica(NorthAmerica::SiliconValley)),
            "Toronto" => Ok(Region::NorthAmerica(NorthAmerica::Toronto)),
            "Miami" => Ok(Region::NorthAmerica(NorthAmerica::Miami)),
            "Mexico City" => Ok(Region::NorthAmerica(NorthAmerica::MexicoCity)),
            "Honolulu" => Ok(Region::NorthAmerica(NorthAmerica::Honolulu)),

            "Sao Paulo" => Ok(Region::SouthAmerica(SouthAmerica::SaoPaulo)),
            "Santiago" => Ok(Region::SouthAmerica(SouthAmerica::Santiago)),

            "Johannesburg" => Ok(Region::Africa(Africa::Johannesburg)),

            _ => Err(()),  // return error for unknown city
        }
    }
}

impl fmt::Display for Region {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Region::Asia(city) => match city {
				Asia::Tokyo => write!(f, "Tokyo"),
				Asia::Osaka => write!(f, "Osaka"),
				Asia::Seoul => write!(f, "Seoul"),
				Asia::Singapore => write!(f, "Singapore"),
				Asia::Mumbai => write!(f, "Mumbai"),
				Asia::TelAviv => write!(f, "Tel Aviv"),
				Asia::Bangalore => write!(f, "Bangalore"),
				Asia::Delhi => write!(f, "Delhi"),
				_ => write!(f, "Unknown city in Asia"),
			},
			Region::Australia(city) => match city {
				Australia::Sydney => write!(f, "Sydney"),
				Australia::Melbourne => write!(f, "Melbourne"),
				_ => write!(f, "Unknown city in Australia"),
			},
			Region::Europe(city) => match city {
				Europe::Amsterdam => write!(f, "Amsterdam"),
				Europe::London => write!(f, "London"),
				Europe::Frankfurt => write!(f, "Frankfurt"),
				Europe::Paris => write!(f, "Paris"),
				Europe::Warsaw => write!(f, "Warsaw"),
				Europe::Madrid => write!(f, "Madrid"),
				Europe::Stockholm => write!(f, "Stockholm"),
				_ => write!(f, "Unknown city in Europe"),
			},
			Region::NorthAmerica(city) => match city {
				NorthAmerica::NewJersey => write!(f, "New Jersey"),
				NorthAmerica::Chicago => write!(f, "Chicago"),
				NorthAmerica::Dallas => write!(f, "Dallas"),
				NorthAmerica::Seattle => write!(f, "Seattle"),
				NorthAmerica::LosAngeles => write!(f, "Los Angeles"),
				NorthAmerica::Atlanta => write!(f, "Atlanta"),
				NorthAmerica::SiliconValley => write!(f, "Silicon Valley"),
				NorthAmerica::Toronto => write!(f, "Toronto"),
				NorthAmerica::Miami => write!(f, "Miami"),
				NorthAmerica::MexicoCity => write!(f, "Mexico City"),
				NorthAmerica::Honolulu => write!(f, "Honolulu"),
				_ => write!(f, "Unknown city in North America"),
			},
			Region::SouthAmerica(city) => match city {
				SouthAmerica::SaoPaulo => write!(f, "Sao Paulo"),
				SouthAmerica::Santiago => write!(f, "Santiago"),
				_ => write!(f, "Unknown city in South America"),
			},
			Region::Africa(city) => match city {
				Africa::Johannesburg => write!(f, "Johannesburg"),
				_ => write!(f, "Unknown city in Africa"),
			},
			Region::Unknown => write!(f, "Unknown"),
		}
	}	
}

impl Region {
	fn code(&self) -> String {
		match self {
			Region::Asia(city) => match city {
				Asia::Tokyo => "nrt".to_string(),
				Asia::Osaka => "itm".to_string(),
				Asia::Seoul => "icn".to_string(),
				Asia::Singapore => "sgp".to_string(),
				Asia::Mumbai => "bom".to_string(),
				Asia::TelAviv => "tlv".to_string(),
				Asia::Bangalore => "blr".to_string(),
				Asia::Delhi => "del".to_string(),
			},
			Region::Australia(city) => match city {
				Australia::Sydney => "syd".to_string(),
				Australia::Melbourne => "mel".to_string(),
			},
			Region::Europe(city) => match city {
				Europe::Amsterdam => "ams".to_string(),
				Europe::London => "lhr".to_string(),
				Europe::Frankfurt => "fra".to_string(),
				Europe::Paris => "cdg".to_string(),
				Europe::Warsaw => "waw".to_string(),
				Europe::Madrid => "mad".to_string(),
				Europe::Stockholm => "sto".to_string(),
			},
			Region::NorthAmerica(city) => match city {
				NorthAmerica::NewJersey => "ewr".to_string(),
				NorthAmerica::Chicago => "ord".to_string(),
				NorthAmerica::Dallas => "dfw".to_string(),
				NorthAmerica::Seattle => "sea".to_string(),
				NorthAmerica::LosAngeles => "lax".to_string(),
				NorthAmerica::Atlanta => "atl".to_string(),
				NorthAmerica::SiliconValley => "sjc".to_string(),
				NorthAmerica::Toronto => "yto".to_string(),
				NorthAmerica::Miami => "mia".to_string(),
				NorthAmerica::MexicoCity => "mex".to_string(),
				NorthAmerica::Honolulu => "hnl".to_string(),
			},
			Region::SouthAmerica(city) => match city {
				SouthAmerica::SaoPaulo => "sao".to_string(),
				SouthAmerica::Santiago => "scl".to_string(),
			},
			Region::Africa(city) => match city {
				Africa::Johannesburg => "jnb".to_string(),
			},
			Region::Unknown => "Unknown".to_string(),
		}
	}

	pub fn from_code(code: &str) -> Result<Self, &'static str> {
		match code {
			"nrt" => Ok(Region::Asia(Asia::Tokyo)),
			"itm" => Ok(Region::Asia(Asia::Osaka)),
			"icn" => Ok(Region::Asia(Asia::Seoul)),
			"sgp" => Ok(Region::Asia(Asia::Singapore)),
			"bom" => Ok(Region::Asia(Asia::Mumbai)),
			"tlv" => Ok(Region::Asia(Asia::TelAviv)),
			"blr" => Ok(Region::Asia(Asia::Bangalore)),
			"del" => Ok(Region::Asia(Asia::Delhi)),
			"syd" => Ok(Region::Australia(Australia::Sydney)),
			"mel" => Ok(Region::Australia(Australia::Melbourne)),
			"ams" => Ok(Region::Europe(Europe::Amsterdam)),
			"lhr" => Ok(Region::Europe(Europe::London)),
			"fra" => Ok(Region::Europe(Europe::Frankfurt)),
			"cdg" => Ok(Region::Europe(Europe::Paris)),
			"waw" => Ok(Region::Europe(Europe::Warsaw)),
			"mad" => Ok(Region::Europe(Europe::Madrid)),
			"sto" => Ok(Region::Europe(Europe::Stockholm)),
			"ewr" => Ok(Region::NorthAmerica(NorthAmerica::NewJersey)),
			"ord" => Ok(Region::NorthAmerica(NorthAmerica::Chicago)),
			"dfw" => Ok(Region::NorthAmerica(NorthAmerica::Dallas)),
			"sea" => Ok(Region::NorthAmerica(NorthAmerica::Seattle)),
			"lax" => Ok(Region::NorthAmerica(NorthAmerica::LosAngeles)),
			"atl" => Ok(Region::NorthAmerica(NorthAmerica::Atlanta)),
			"sjc" => Ok(Region::NorthAmerica(NorthAmerica::SiliconValley)),
			"yto" => Ok(Region::NorthAmerica(NorthAmerica::Toronto)),
			"mia" => Ok(Region::NorthAmerica(NorthAmerica::Miami)),
			"mex" => Ok(Region::NorthAmerica(NorthAmerica::MexicoCity)),
			"hnl" => Ok(Region::NorthAmerica(NorthAmerica::Honolulu)),
			"sao" => Ok(Region::SouthAmerica(SouthAmerica::SaoPaulo)),
			"scl" => Ok(Region::SouthAmerica(SouthAmerica::Santiago)),
			"jnb" => Ok(Region::Africa(Africa::Johannesburg)),
			"unknown" => Ok(Region::Unknown),
			_ => Err("Unknown region code"),
		}
	}
}

impl Serialize for Region {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		serializer.serialize_str(&self.code())
	}
}

struct RegionVisitor;

impl<'de> Visitor<'de> for RegionVisitor {
	type Value = Region;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("a region code string")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		Region::from_code(v).map_err(|err| de::Error::custom(err))
	}
}

impl<'de> Deserialize<'de> for Region {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(RegionVisitor)
	}
}
