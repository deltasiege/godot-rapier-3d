use bincode::config::standard;
use bincode::serde::{decode_from_slice, encode_to_vec};
use godot::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub fn encode_or_none<T>(value: &T) -> Option<Vec<u8>>
where
    T: Serialize + Debug + Clone,
{
    match encode_to_vec(value, standard()) {
        Ok(encoded) => Some(encoded),
        Err(_) => {
            log::error!("Failed to encode value: {:?}", value);
            None
        }
    }
}

pub fn decode_or_none<T>(data: &[u8]) -> Option<T>
where
    T: DeserializeOwned + Debug,
{
    match decode_from_slice(data, standard()) {
        Ok((decoded, _)) => Some(decoded),
        Err(_) => {
            log::error!("Failed to decode data: {:?}", data);
            None
        }
    }
}

pub fn serialize_variant(variant: &Variant) -> Option<Vec<u8>> {
    match variant.get_type() {
        VariantType::BOOL => encode_or_none(&variant.to::<bool>()),
        VariantType::INT => encode_or_none(&variant.to::<i64>()),
        VariantType::FLOAT => encode_or_none(&variant.to::<f64>()),
        VariantType::STRING => encode_or_none(&variant.to::<GString>()),
        VariantType::VECTOR2 => encode_or_none(&variant.to::<Vector2>()),
        VariantType::VECTOR3 => encode_or_none(&variant.to::<Vector3>()),
        _ => {
            log::error!(
                "Can't serialize unsupported variant type: {:?}",
                variant.get_type()
            );
            None
        }
    }
}

pub fn deserialize_variant(data: &[u8], variant_type: VariantType) -> Option<Variant> {
    match data {
        [] => None,
        _ => match variant_type {
            VariantType::BOOL => decode_or_none::<bool>(data).map(|v| Variant::from(v)),
            VariantType::INT => decode_or_none::<i64>(data).map(|v| Variant::from(v)),
            VariantType::FLOAT => decode_or_none::<f64>(data).map(|v| Variant::from(v)),
            VariantType::STRING => decode_or_none::<GString>(data).map(|v| Variant::from(v)),
            VariantType::VECTOR2 => decode_or_none::<Vector2>(data).map(|v| Variant::from(v)),
            VariantType::VECTOR3 => decode_or_none::<Vector3>(data).map(|v| Variant::from(v)),
            _ => {
                log::error!(
                    "Can't deserialize unsupported variant type: {:?}",
                    variant_type
                );
                None
            }
        },
    }
}
