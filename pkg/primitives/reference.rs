use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct FileReference {
    #[serde(alias = "fileID")]
    pub file_id: i64,
    #[serde(default, deserialize_with = "deserialize_option_string_or_float")]
    pub guid: Option<String>,
}

fn deserialize_option_string_or_float<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrFloat;

    impl<'de> Visitor<'de> for StringOrFloat {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a float")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            Ok(Some(value.to_owned()))
        }

        fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
            Ok(Some(value))
        }

        fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_f32<E: de::Error>(self, _: f32) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrFloat)
}
