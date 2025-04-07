use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

use crate::utils::{decode_or_none, deserialize_variant, encode_or_none, serialize_variant};

#[derive(GodotClass)]
#[class(base = Node)]
/// Base class that must be overriden by GDScript
/// User must define:
/// - all_inputs - an array of all actions that can be used by rollback nodes
/// - get_input - a function that returns some value for every possible action specified in all_inputs
/// Responsible collecting and serializing/deserializing inputs
pub struct GR3DInputAdapter {
    pub all_inputs: Vec<String>,
    base: Base<Node>,
}

#[godot_api]
impl INode for GR3DInputAdapter {
    fn init(base: Base<Node>) -> Self {
        Self {
            all_inputs: Vec::new(),
            base,
        }
    }
}

#[godot_api]
impl GR3DInputAdapter {
    #[func(virtual)]
    pub fn get_input(&self) -> Variant {
        log::error!(
            "UNIMPLEMENTED: get_input on InputAdapter: {:?}",
            self.base().get_name()
        );
        Variant::nil()
    }

    fn get_inputs(&self) -> HashMap<String, Variant> {
        let mut inputs = HashMap::default();
        for input_key in self.all_inputs.iter() {
            let input_value = self.get_input();
            inputs.insert(input_key.clone(), input_value);
        }

        inputs
    }

    fn get_ser_inputs(&self) -> Vec<u8> {
        let mut sorted_inputs: Vec<_> = self.get_inputs().into_iter().collect();
        sorted_inputs.sort_by(|a, b| a.0.cmp(&b.0));

        let values: Vec<InputValue> = sorted_inputs
            .iter()
            .filter_map(|(_, v)| InputValue::try_from_variant(v))
            .collect();

        encode_or_none(&values).unwrap_or_default()
    }

    fn deserialize_inputs(&self, ser_inputs: Vec<u8>) -> HashMap<String, Variant> {
        let mut sorted_inputs: Vec<_> = self.get_inputs().into_iter().collect();
        sorted_inputs.sort_by(|a, b| a.0.cmp(&b.0));

        let decoded: Vec<InputValue> = decode_or_none(ser_inputs.as_slice()).unwrap_or_default();
        let mut inputs = HashMap::default();

        for (i, input) in decoded.iter().enumerate() {
            if let Some(variant) = input.try_to_variant() {
                let key = match sorted_inputs.get(i).map(|(k, _)| k.clone()) {
                    Some(k) => k,
                    None => {
                        log::error!(
                            "Error while deserializing inputs. Key not found for index {}: {:?}",
                            i,
                            sorted_inputs
                        );
                        continue;
                    }
                };
                inputs.insert(key, variant);
            }
        }
        inputs
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InputValue {
    value: Vec<u8>,
    value_type: SerVariantType,
}

impl InputValue {
    fn try_from_variant(value: &Variant) -> Option<Self> {
        let value_type = SerVariantType::try_from(value.get_type()).ok()?;
        let value = serialize_variant(&value)?;
        Some(Self { value, value_type })
    }

    fn try_to_variant(&self) -> Option<Variant> {
        deserialize_variant(self.value.as_slice(), self.value_type.clone().into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum SerVariantType {
    BOOL,
    INT,
    FLOAT,
    VECTOR2,
    VECTOR3,
}

impl TryFrom<VariantType> for SerVariantType {
    type Error = String;

    fn try_from(value: VariantType) -> Result<Self, Self::Error> {
        match value {
            VariantType::BOOL => Ok(Self::BOOL),
            VariantType::INT => Ok(Self::INT),
            VariantType::FLOAT => Ok(Self::FLOAT),
            VariantType::VECTOR2 => Ok(Self::VECTOR2),
            VariantType::VECTOR3 => Ok(Self::VECTOR3),
            _ => {
                log::error!("Unsupported variant type: {:?}", value);
                Err(format!("Unsupported variant type: {:?}", value))
            }
        }
    }
}

impl Into<VariantType> for SerVariantType {
    fn into(self) -> VariantType {
        match self {
            Self::BOOL => VariantType::BOOL,
            Self::INT => VariantType::INT,
            Self::FLOAT => VariantType::FLOAT,
            Self::VECTOR2 => VariantType::VECTOR2,
            Self::VECTOR3 => VariantType::VECTOR3,
        }
    }
}
