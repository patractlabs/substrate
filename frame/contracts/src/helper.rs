use serde::{ser, de, Deserialize};
use sp_std::result::Result;
use sp_runtime::{traits::AtLeast32BitUnsigned, SaturatedConversion};

/// Number string serialization/deserialization
pub mod serde_num_str {
	use super::*;

	/// A serializer that encodes the number as a string
	pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
			T: AtLeast32BitUnsigned + Copy,
	{
		let v = (*value).saturated_into::<u128>();
		serializer.serialize_str(&v.to_string())
	}

	/// A deserializer that decodes a string to the number.
	pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
		where
			D: de::Deserializer<'de>,
			T: AtLeast32BitUnsigned,
	{
		let data = String::deserialize(deserializer)?;
		let num = data.parse::<u128>()
			.map_err(|_| de::Error::custom("Parse from string failed"))?;
		Ok(num.saturated_into())
	}
}

/// Option<Number> string serialization/deserialization
pub mod serde_opt_num_str {
	use super::*;

	/// A serializer that encodes the number as a string
	pub fn serialize<S, T>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
		where
			S: ser::Serializer,
			T: AtLeast32BitUnsigned + Copy,
	{
		match value {
			Some(ref value) => {
				let v = (*value).saturated_into::<u128>();
				serializer.serialize_str(&v.to_string())
			},
			None => serializer.serialize_none(),
		}
	}

	/// A deserializer that decodes a string to the number.
	pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
		where
			D: de::Deserializer<'de>,
			T: AtLeast32BitUnsigned,
	{
		let data: Option<String> = Deserialize::deserialize(deserializer)?;
		Ok(match data {
			Some(data) => {
				let num = data.parse::<u128>()
					.map_err(|_| de::Error::custom("Parse from string failed"))?;
				Some(num.saturated_into())
			}
			None => None,
		})
	}
}
