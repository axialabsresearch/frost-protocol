#![cfg(feature = "std")]

use parity_scale_codec::{Encode, Decode};
use scale_info::TypeInfo;

use crate::state::{StateTransition, StateProof};
use crate::message::FrostMessage;

// Implement Substrate traits for StateTransition
impl Encode for StateTransition {
    fn encode(&self) -> Vec<u8> {
        // Use serde to encode to JSON bytes first
        serde_json::to_vec(self).unwrap_or_default()
    }
}

impl Decode for StateTransition {
    fn decode<I: parity_scale_codec::Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let bytes: Vec<u8> = Vec::decode(input)?;
        serde_json::from_slice(&bytes)
            .map_err(|_| parity_scale_codec::Error::from("Failed to decode StateTransition"))
    }
}

impl TypeInfo for StateTransition {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("StateTransition", module_path!()))
            .composite(scale_info::build::Fields::named()
                .field(|f| f.ty::<Vec<u8>>().name("encoded").docs(&["Encoded state transition data"]))
            )
    }
}

// Implement Substrate traits for StateProof
impl Encode for StateProof {
    fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

impl Decode for StateProof {
    fn decode<I: parity_scale_codec::Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let bytes: Vec<u8> = Vec::decode(input)?;
        serde_json::from_slice(&bytes)
            .map_err(|_| parity_scale_codec::Error::from("Failed to decode StateProof"))
    }
}

impl TypeInfo for StateProof {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("StateProof", module_path!()))
            .composite(scale_info::build::Fields::named()
                .field(|f| f.ty::<Vec<u8>>().name("encoded").docs(&["Encoded state proof data"]))
            )
    }
}

// Implement Substrate traits for FrostMessage
impl Encode for FrostMessage {
    fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }
}

impl Decode for FrostMessage {
    fn decode<I: parity_scale_codec::Input>(input: &mut I) -> Result<Self, parity_scale_codec::Error> {
        let bytes: Vec<u8> = Vec::decode(input)?;
        serde_json::from_slice(&bytes)
            .map_err(|_| parity_scale_codec::Error::from("Failed to decode FrostMessage"))
    }
}

impl TypeInfo for FrostMessage {
    type Identity = Self;

    fn type_info() -> scale_info::Type {
        scale_info::Type::builder()
            .path(scale_info::Path::new("FrostMessage", module_path!()))
            .composite(scale_info::build::Fields::named()
                .field(|f| f.ty::<Vec<u8>>().name("encoded").docs(&["Encoded message data"]))
            )
    }
} 