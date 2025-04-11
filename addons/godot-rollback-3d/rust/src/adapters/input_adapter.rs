use godot::prelude::*;
use rapier3d::parry::utils::hashmap::HashMap;
use serde::{Deserialize, Serialize};

use crate::interface::GR3D;
use crate::types::*;
use crate::utils::*;

#[derive(GodotClass)]
#[class(base = Node)]
/// Base class that must be overriden by GDScript
/// Responsible collecting and serializing/deserializing inputs
/// User must define functions:
/// - get_input_list
/// - get_input
pub struct GR3DInputAdapter {
    base: Base<Node>,
}

#[godot_api]
impl INode for GR3DInputAdapter {
    fn init(base: Base<Node>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GR3DInputAdapter {
    #[func(virtual)]
    /// Must be provided. Returns a constant array of all actions that can be used by rollback nodes. Must not change at runtime.
    fn get_input_list(&self) -> Vec<GString> {
        log::error!(
            "UNIMPLEMENTED: get_input_list on InputAdapter: {:?}",
            self.base().get_name()
        );
        Vec::new()
    }

    #[func(virtual)]
    /// Must be provided. Returns some variant value for every possible action specified in all_inputs.
    fn get_input(&self, _input_key: GString) -> Variant {
        log::error!(
            "UNIMPLEMENTED: get_input on InputAdapter: {:?}",
            self.base().get_name()
        );
        Variant::nil()
    }

    /// Returns all current inputs as an InputMap.
    pub fn get_inputs(&mut self) -> InputMap {
        // Doesnt call overriden func
        // self.get_input_list()
        //     .iter()
        //     .map(|input_key| (input_key.clone(), self.get_input(input_key.clone())))
        //     .collect();

        let input_list = self
            .base_mut()
            .call("get_input_list", &[])
            .to::<Vec<GString>>();

        input_list
            .iter()
            .map(|input_key| {
                (
                    input_key.clone(),
                    self.base_mut().call("get_input", &[input_key.to_variant()]),
                )
            })
            .collect()
    }

    /// Returns all current inputs as a serialized byte array.
    pub fn get_ser_inputs(&mut self) -> Vec<u8> {
        let mut sorted_inputs: Vec<_> = self.get_inputs().into_iter().collect();
        sorted_inputs.sort_by(|a, b| a.0.cmp(&b.0));

        let values: Vec<InputValue> = sorted_inputs
            .iter()
            .filter_map(|(_, v)| InputValue::try_from_variant(v))
            .collect();

        encode_or_none(&values).unwrap_or_default()
    }

    /// Deserializes the inputs from a byte array and returns an InputMap.
    pub fn deserialize_inputs(&mut self, ser_inputs: &Vec<u8>) -> InputMap {
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

/// Attach a input adapter to the GR3D instance and connect all signals to the GR3D singleton.
pub fn attach_input_adapter(gr3d: &mut GR3D, adapter: Gd<GR3DInputAdapter>) {
    log::debug!("Attaching InputAdapter: {:?}", adapter);
    gr3d.network.local_peer.input_adapter = Some(adapter.clone());
}

/// Detach the input adapter and disconnect all signals from the GR3D singleton.
pub fn detach_input_adapter(gr3d: &mut GR3D) {
    log::debug!(
        "Detaching InputAdapter: {:?}",
        gr3d.network.local_peer.input_adapter
    );
    gr3d.network.local_peer.input_adapter = None;
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
