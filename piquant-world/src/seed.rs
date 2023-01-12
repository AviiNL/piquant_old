use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum SeedType {
    Random,
    Value(u32),
    FromString(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seed(u32);

impl Seed {
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl From<SeedType> for Seed {
    fn from(seed: SeedType) -> Self {
        match seed {
            SeedType::Random => Seed(rand::random()),
            SeedType::Value(val) => Seed(val),
            SeedType::FromString(val) => {
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                val.hash(&mut hasher);
                Seed(hasher.finish() as u32)
            }
        }
    }
}

impl Into<u32> for Seed {
    fn into(self) -> u32 {
        self.0.clone()
    }
}

// use std::hash::{Hash, Hasher};

// use serde::{Deserialize, Serialize};

// #[derive(Debug, Clone)]
// pub enum Seed {
//     Random,
//     Value(u32),
//     FromString(String),
// }

impl Serialize for SeedType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Random => serializer.serialize_str(""),
            Self::Value(val) => serializer.serialize_u32(*val),
            Self::FromString(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SeedType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SeedVisitor;

        impl<'de> serde::de::Visitor<'de> for SeedVisitor {
            type Value = SeedType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or integer")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SeedType::Random)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if v.len() == 0 {
                    return Ok(SeedType::Random);
                }

                // try parsing as a number
                if let Ok(val) = v.parse::<u32>() {
                    return Ok(SeedType::Value(val));
                }

                Ok(SeedType::FromString(v.to_string()))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SeedType::Value(v as u32))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(SeedType::Value(v as u32))
            }
        }

        deserializer.deserialize_any(SeedVisitor)
    }
}

// impl Seed {
//     pub fn random() -> Self {
//         Self::Value(rand::random())
//     }
// }

// impl From<u32> for Seed {
//     fn from(seed: u32) -> Self {
//         Self::Value(seed)
//     }
// }

// impl From<Option<Seed>> for Seed {
//     fn from(seed: Option<Seed>) -> Self {
//         seed.unwrap_or_else(Seed::random)
//     }
// }

// impl Into<u32> for Seed {
//     fn into(self) -> u32 {
//         match self {
//             Seed::Random => rand::random(),
//             Seed::Value(val) => val,
//             Seed::FromString(val) => {
//                 let mut hasher = std::collections::hash_map::DefaultHasher::new();
//                 val.hash(&mut hasher);
//                 hasher.finish() as u32
//             }
//         }
//     }
// }
