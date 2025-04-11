use bincode::config::standard;
use bincode::error::EncodeError;
use bincode::serde::{decode_from_slice, encode_to_vec};
use godot::prelude::*;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;

/// Wrapper to abstract away configuration details of bincode.
pub fn encode(value: &(impl Serialize + Debug)) -> Result<Vec<u8>, EncodeError> {
    encode_to_vec(value, standard())
}

/// Wraps the given data in a `PackedByteArray` if it is `Some`, otherwise returns an empty `PackedByteArray`.
pub fn try_wrap_bytes(data: Option<Vec<u8>>) -> PackedByteArray {
    match data {
        Some(data) => PackedByteArray::from(data),
        None => PackedByteArray::new(),
    }
}

/// Wrapper to handle error logging and return an Option<bytes>.
pub fn encode_or_none<T>(value: &T) -> Option<Vec<u8>>
where
    T: Serialize + Debug + Clone,
{
    match encode(value) {
        Ok(encoded) => Some(encoded),
        Err(_) => {
            log::error!("Failed to encode value: {:?}", value);
            None
        }
    }
}

/// Wrapper to handle error logging and return an Option<T>.
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

/// Encodes the given value to a `PackedByteArray`.
pub fn encode_to_packed_byte_array<T>(value: &T) -> PackedByteArray
where
    T: Serialize + Debug,
{
    match encode(value) {
        Ok(encoded) => PackedByteArray::from(encoded),
        Err(e) => {
            log::error!(
                "Failed to encode value to PackedByteArray: {:?}. Error: {}",
                value,
                e
            );
            PackedByteArray::new()
        }
    }
}

/// Decodes the given `PackedByteArray` to a value of type `T`.
pub fn decode_from_packed_byte_array<T>(data: &PackedByteArray) -> Option<T>
where
    T: DeserializeOwned + Debug,
{
    if data.len() == 0 {
        return None;
    }

    let data_slice = data.as_slice();
    match decode_from_slice(data_slice, standard()) {
        Ok((decoded, _)) => Some(decoded),
        Err(e) => {
            log::error!("Failed to decode PackedByteArray: {}", e);
            None
        }
    }
}

/// Encode the given variant to bytes if it is able to be serialized.
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

/// Deserialize the given bytes to a `Variant` based on the provided `VariantType`.
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
