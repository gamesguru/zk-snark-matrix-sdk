#![no_main]
sp1_zkvm::entrypoint!(main);

use ruma_common::{
    CanonicalJsonObject, OwnedEventId, OwnedRoomId, OwnedUserId, RoomId, RoomVersionId, UserId,
};
use ruma_events::TimelineEventType;
use ruma_state_res::Event;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

extern crate alloc;
use alloc::vec::Vec;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GuestEvent {
    pub event: CanonicalJsonObject,
    pub content: Box<serde_json::value::RawValue>,
    pub event_id: OwnedEventId,
    pub room_id: OwnedRoomId,
    pub sender: OwnedUserId,
    pub event_type: TimelineEventType,
    pub prev_events: Vec<OwnedEventId>,
    pub auth_events: Vec<OwnedEventId>,
}

impl Event for GuestEvent {
    type Id = OwnedEventId;

    fn event_id(&self) -> &Self::Id {
        &self.event_id
    }

    fn room_id(&self) -> Option<&RoomId> {
        Some(&self.room_id)
    }

    fn sender(&self) -> &UserId {
        &self.sender
    }

    fn origin_server_ts(&self) -> ruma_common::MilliSecondsSinceUnixEpoch {
        let val = self
            .event
            .get("origin_server_ts")
            .expect("missing origin_server_ts");
        serde_json::from_value(val.clone().into()).expect("invalid origin_server_ts")
    }

    fn event_type(&self) -> &TimelineEventType {
        &self.event_type
    }

    fn content(&self) -> &serde_json::value::RawValue {
        &self.content
    }

    fn state_key(&self) -> Option<&str> {
        let val = self.event.get("state_key").expect("missing state_key");
        val.as_str()
    }

    fn prev_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
        Box::new(self.prev_events.iter())
    }

    fn auth_events(&self) -> Box<dyn DoubleEndedIterator<Item = &Self::Id> + '_> {
        Box::new(self.auth_events.iter())
    }

    fn redacts(&self) -> Option<&Self::Id> {
        None
    }

    fn rejected(&self) -> bool {
        false
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DAGMergeInput {
    pub room_version: RoomVersionId,
    pub events: Vec<GuestEvent>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DAGMergeOutput {
    pub resolved_state_hash: [u8; 32],
}

pub fn main() {
    // Read the input from the Host (the world-state hint containing events)
    let input: DAGMergeInput = sp1_zkvm::io::read();

    // In this iteration, we iterate over the raw events and verify their internal structure.
    // Full ruma_state_res::resolve invocation will go here once the State Resolution
    // traits are implemented for the Guest events.
    let mut hasher = Sha256::new();

    for event in input.events {
        // Simple hash commitment for consistency verification
        // (This will be replaced by the signature + hash resolution logic)
        let json_str = serde_json::to_string(&event).expect("Failed to serialize Ruma event!");
        hasher.update(json_str.as_bytes());
    }

    let expected_hash: [u8; 32] = hasher.finalize().into();

    let output = DAGMergeOutput {
        resolved_state_hash: expected_hash,
    };

    sp1_zkvm::io::commit(&output);
}
