// use godot::prelude::*;
// use rapier3d::parry::utils::hashmap::HashMap;
// use serde::{Deserialize, Serialize};

// use crate::interface::GR3D;
// use crate::types::*;
// use crate::utils::*;

// #[derive(GodotClass)]
// #[class(base = Node)]
// /// Base class that must be overriden by GDScript
// /// Responsible collecting and serializing/deserializing inputs
// /// User must define functions:
// /// - get_input_list
// /// - get_input
// /// - get_default
// pub struct GR3DInputAdapter {
//     base: Base<Node>,
// }

// #[godot_api]
// impl INode for GR3DInputAdapter {
//     fn init(base: Base<Node>) -> Self {
//         Self { base }
//     }
// }

// #[godot_api]
// impl GR3DInputAdapter {
//     #[func(virtual)]
//     /// Must be provided. Returns a constant array of all actions that can be used by rollback nodes. Must not change at runtime.
//     fn get_input_list(&self) -> Vec<GString> {
//         log::error!(
//             "UNIMPLEMENTED: get_input_list on InputAdapter: {:?}",
//             self.base().get_name()
//         );
//         Vec::new()
//     }

//     /// Returns result of potentially overriden get_input_list function.
//     pub fn get_overriden_input_list(&mut self) -> Vec<GString> {
//         self.base_mut()
//             .call("get_input_list", &[])
//             .to::<Vec<GString>>()
//     }

//     #[func(virtual)]
//     /// Must be provided. Returns some variant value for every possible action specified in all_inputs.
//     fn get_input(&self, _input_key: GString, _node_state: Dictionary) -> Variant {
//         log::error!(
//             "UNIMPLEMENTED: get_input on InputAdapter: {:?}",
//             self.base().get_name()
//         );
//         Variant::nil()
//     }

//     /// Returns result of potentially overriden get_input function.
//     pub fn get_overriden_input(&mut self, input_key: &GString, node_state: &Dictionary) -> Variant {
//         self.base_mut().call(
//             "get_input",
//             &[input_key.to_variant(), node_state.to_variant()],
//         )
//     }

//     #[func(virtual)]
//     /// Must be provided. Returns a default variant value for every possible action specified in all_inputs.
//     fn get_default(&self, _input_key: GString) -> Variant {
//         log::error!(
//             "UNIMPLEMENTED: get_default on InputAdapter: {:?}",
//             self.base().get_name()
//         );
//         Variant::nil()
//     }

//     #[func(virtual)]
//     /// Optional. Provides a predicted input based on the previous input. Defaults to returning the previous input.
//     fn get_predicted_input(&self, _input_key: GString, previous_input: Variant) -> Variant {
//         previous_input
//     }

//     /// Returns result of potentially overriden get_predicted_input function.
//     pub fn get_overriden_predicted_input(
//         &mut self,
//         input_key: &GString,
//         previous_input: Variant,
//     ) -> Variant {
//         self.base_mut().call(
//             "get_predicted_input",
//             &[input_key.to_variant(), previous_input],
//         )
//     }

//     /// Returns all current inputs as an InputMap.
//     pub fn get_inputs(&mut self, node_state: &Dictionary) -> InputMap {
//         let input_list = self.get_overriden_input_list();
//         input_list
//             .iter()
//             .map(|input_key| {
//                 (
//                     input_key.clone(),
//                     self.get_overriden_input(input_key, node_state),
//                 )
//             })
//             .collect()
//     }

//     /// Returns all predicted inputs based on a previous InputMap.
//     // TODO does predicted inputs need node_state?
//     pub fn get_predicted_inputs(&mut self, previous_inputs: &InputMap) -> InputMap {
//         let mut predicted_inputs: InputMap = HashMap::default();

//         for (input_key, previous_input) in previous_inputs.iter() {
//             let predicted_input =
//                 self.get_overriden_predicted_input(input_key, previous_input.clone());
//             predicted_inputs.insert(input_key.clone(), predicted_input);
//         }

//         previous_inputs.clone()
//     }

//     /// Returns all current inputs as a serialized byte array.
//     pub fn get_ser_inputs(&mut self, node_state: &Dictionary) -> Vec<u8> {
//         let inputs = self.get_inputs(node_state);
//         serialize_inputs(&inputs)
//     }

//     /// Returns a deserialized InputMap from a serialized byte array,
//     /// using a current copy of the input_list.
//     pub fn deserialize_inputs(&mut self, ser_inputs: &Vec<u8>) -> InputMap {
//         let mut input_keys = self.get_overriden_input_list();
//         input_keys.sort_by(|a, b| a.cmp(b));

//         let decoded: Vec<InputValue> = decode_or_none(ser_inputs.as_slice()).unwrap_or_default();
//         let mut inputs = HashMap::default();

//         for (i, input) in decoded.iter().enumerate() {
//             if let Some(variant) = input.try_to_variant() {
//                 let key = match input_keys.get(i) {
//                     Some(k) => k.clone(),
//                     None => {
//                         log::error!(
//                         "Error while deserializing inputs. Index {} not found. Provided keys: {:?}",
//                         i,
//                         input_keys
//                     );
//                         continue;
//                     }
//                 };
//                 inputs.insert(key, variant);
//             }
//         }
//         inputs
//     }
// }

// /// Returns the given InpuMap as a serialized byte array.
// pub fn serialize_inputs(inputs: &InputMap) -> Vec<u8> {
//     let mut sorted_inputs: Vec<_> = inputs.into_iter().collect();
//     sorted_inputs.sort_by(|a, b| a.0.cmp(&b.0));

//     let values: Vec<InputValue> = sorted_inputs
//         .iter()
//         .filter_map(|(_, v)| InputValue::try_from_variant(v))
//         .collect();

//     encode_or_none(&values).unwrap_or_default()
// }

// /// Returns a hash of the given InputMap.
// pub fn get_input_hash(inputs: &InputMap) -> u64 {
//     let ser_inputs = serialize_inputs(inputs);
//     get_hash(&ser_inputs)
// }

// /// Attach a input adapter to the GR3D instance and connect all signals to the GR3D singleton.
// pub fn attach_input_adapter(gr3d: &mut GR3D, adapter: Gd<GR3DInputAdapter>) {
//     log::debug!("Attaching InputAdapter: {:?}", adapter);
//     gr3d.network.local_peer.input_adapter = Some(adapter.clone());
// }

// /// Detach the input adapter and disconnect all signals from the GR3D singleton.
// pub fn detach_input_adapter(gr3d: &mut GR3D) {
//     log::debug!(
//         "Detaching InputAdapter: {:?}",
//         gr3d.network.local_peer.input_adapter
//     );
//     gr3d.network.local_peer.input_adapter = None;
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// struct InputValue {
//     value: Vec<u8>,
//     value_type: SerdeVarType,
// }

// impl InputValue {
//     fn try_from_variant(value: &Variant) -> Option<Self> {
//         let value_type = SerdeVarType::try_from(value.get_type()).ok()?;
//         let value = serialize_variant(&value)?;
//         Some(Self { value, value_type })
//     }

//     fn try_to_variant(&self) -> Option<Variant> {
//         deserialize_variant(self.value.as_slice(), self.value_type.clone().into())
//     }
// }
