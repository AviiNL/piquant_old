use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Seed {
    Random,
    Value(u32),
    FromString(String),
}

impl Serialize for Seed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Seed::Random => serializer.serialize_str(""),
            Seed::Value(val) => serializer.serialize_u32(*val),
            Seed::FromString(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SeedVisitor;

        impl<'de> serde::de::Visitor<'de> for SeedVisitor {
            type Value = Seed;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or integer")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Seed::Random)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() == 0 {
                    return Ok(Seed::Random);
                }

                // try parsing as a number
                if let Ok(val) = v.parse::<u32>() {
                    return Ok(Seed::Value(val));
                }

                Ok(Seed::FromString(v.to_string()))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Seed::Value(v as u32))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Seed::Value(v as u32))
            }
        }

        deserializer.deserialize_any(SeedVisitor)
    }
}

impl Seed {
    pub fn random() -> Self {
        Self::Value(rand::random())
    }
}

impl From<u32> for Seed {
    fn from(seed: u32) -> Self {
        Self::Value(seed)
    }
}

impl From<Option<Seed>> for Seed {
    fn from(seed: Option<Seed>) -> Self {
        seed.unwrap_or_else(Seed::random)
    }
}

impl Into<u32> for Seed {
    fn into(self) -> u32 {
        match self {
            Seed::Random => rand::random(),
            Seed::Value(val) => val,
            Seed::FromString(val) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                val.hash(&mut hasher);
                hasher.finish() as u32
            }
        }
    }
}
