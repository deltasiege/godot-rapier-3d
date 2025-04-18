use bincode::config::standard;
use bincode::error::EncodeError;
use bincode::serde::{decode_from_slice, encode_to_vec};
use godot::prelude::*;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::utils::variant_to;

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

/// A subset of Variant types that can be serialized and deserialized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerdeVar {
    Int(i64),
    Float(f64),
    String(GString),
    Bool(bool),
    Color(Color),
    Vector2(Vector2),
    Vector2i(Vector2i),
    Vector3(Vector3),
    Vector3i(Vector3i),
    Basis(Basis),
    Transform2D(Transform2D),
    Transform3D(Transform3D),
}

impl SerdeVar {
    pub fn to_variant(&self) -> Variant {
        match self {
            SerdeVar::Int(i) => Variant::from(*i),
            SerdeVar::Float(f) => Variant::from(*f),
            SerdeVar::String(s) => Variant::from(s.clone()),
            SerdeVar::Bool(b) => Variant::from(*b),
            SerdeVar::Color(c) => Variant::from(*c),
            SerdeVar::Vector2(v) => Variant::from(*v),
            SerdeVar::Vector2i(v) => Variant::from(*v),
            SerdeVar::Vector3(v) => Variant::from(*v),
            SerdeVar::Vector3i(v) => Variant::from(*v),
            SerdeVar::Basis(b) => Variant::from(*b),
            SerdeVar::Transform2D(t) => Variant::from(*t),
            SerdeVar::Transform3D(t) => Variant::from(*t),
        }
    }

    pub fn from_variant(v: Variant) -> Option<Self> {
        match v.get_type() {
            VariantType::NIL => None,
            VariantType::INT => Some(SerdeVar::Int(variant_to::<i64>(&v)?)),
            VariantType::FLOAT => Some(SerdeVar::Float(variant_to::<f64>(&v)?)),
            VariantType::STRING => Some(SerdeVar::String(variant_to::<GString>(&v)?)),
            VariantType::BOOL => Some(SerdeVar::Bool(variant_to::<bool>(&v)?)),
            VariantType::COLOR => Some(SerdeVar::Color(variant_to::<Color>(&v)?)),
            VariantType::VECTOR2 => Some(SerdeVar::Vector2(variant_to::<Vector2>(&v)?)),
            VariantType::VECTOR2I => Some(SerdeVar::Vector2i(variant_to::<Vector2i>(&v)?)),
            VariantType::VECTOR3 => Some(SerdeVar::Vector3(variant_to::<Vector3>(&v)?)),
            VariantType::VECTOR3I => Some(SerdeVar::Vector3i(variant_to::<Vector3i>(&v)?)),
            VariantType::BASIS => Some(SerdeVar::Basis(variant_to::<Basis>(&v)?)),
            VariantType::TRANSFORM2D => Some(SerdeVar::Transform2D(variant_to::<Transform2D>(&v)?)),
            VariantType::TRANSFORM3D => Some(SerdeVar::Transform3D(variant_to::<Transform3D>(&v)?)),
            _ => {
                log::error!("Variant type: {:?} is not serializable", v.get_type());
                return None;
            }
        }
    }
}
