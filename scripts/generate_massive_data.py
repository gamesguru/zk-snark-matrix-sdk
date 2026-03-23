"""
Generates a massive, synthetic Matrix Room State DAG vector for testing and profiling SP1 STARKs.
"""

import hashlib
import json
import random
import sys
import time

NUM_EVENTS = 1000000


def sha256_hash(data_str):
    """Computes a SHA-256 hex digest for simulating algorithmic cryptographic identifiers."""
    return hashlib.sha256(data_str.encode("utf-8")).hexdigest()


print(f"Generating {NUM_EVENTS} synthetic Matrix state events...", file=sys.stderr)

events = []
ROOM_ID = "!massive_test_room:example.com"

# Create initial event
events.append(
    {
        "event_id": "$00000-m-room-create",
        "room_id": ROOM_ID,
        "sender": "@creator:example.com",
        "type": "m.room.create",
        "content": {"creator": "@creator:example.com", "room_version": "10"},
        "state_key": "",
        "origin_server_ts": int(time.time() * 1000) - 10000000,
        "prev_events": [],
        "auth_events": [],
    }
)

event_types = [
    "m.room.member",
    "m.room.message",
    "m.room.power_levels",
    "m.room.join_rules",
]
members = [f"@user_{i}:example.com" for i in range(100)]

for i in range(1, NUM_EVENTS):
    sender = random.choice(members)
    ev_type = random.choice(event_types)
    ts = events[-1]["origin_server_ts"] + random.randint(1, 1000)

    prev_event_id = events[-1]["event_id"]

    content = {}
    state_key = ""
    if ev_type == "m.room.member":
        content = {"membership": random.choice(["join", "leave", "invite"])}
        state_key = random.choice(members)
    elif ev_type == "m.room.message":
        content = {"body": f"Message {i}", "msgtype": "m.text"}
        # Messages usually lack state_key, but for Res we exclusively mock state events.
        ev_type = "m.room.topic"
        content = {"topic": f"Topic number {i}"}
    elif ev_type == "m.room.power_levels":
        content = {"users": {sender: 100}}
    else:
        content = {"join_rule": "public"}

    # Mock event ID based on hash of index to ensure uniqueness
    event_id = f"${sha256_hash(str(i))[:20]}"

    events.append(
        {
            "event_id": event_id,
            "room_id": ROOM_ID,
            "sender": sender,
            "type": ev_type,
            "content": content,
            "state_key": state_key,
            "origin_server_ts": ts,
            "prev_events": [prev_event_id],
            "auth_events": [events[0]["event_id"]],  # Simplify auth chain
        }
    )

    if i % 10000 == 0:
        print(f"Generated {i} events...", file=sys.stderr)

OUTPUT_FILE = "res/massive_matrix_state.json"
with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
    json.dump(events, f, indent=2)

print(
    f"Success! Generated {NUM_EVENTS} events "
    f"({sys.getsizeof(events)} approx bytes) to {OUTPUT_FILE}",
    file=sys.stderr,
)
