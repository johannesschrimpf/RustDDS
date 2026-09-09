//! A participant's discovery notification receiver must remain usable after
//! both timed-out and successful find_topic calls.

use std::time::Duration;

use rustdds::{policy, DomainParticipant, QosPolicyBuilder, TopicKind};
use serde::Serialize;

#[derive(Serialize)]
struct Sample {
  seq: u32,
}

fn scratch_domain_id(base: u16) -> u16 {
  // Keep domains distinct between these tests and within the RTPS port range.
  base + (std::process::id() % 10) as u16
}

#[test]
fn find_topic_can_be_reused_after_timeout() {
  let dp = DomainParticipant::new(scratch_domain_id(180)).expect("failed to create participant");
  let name = format!("find_topic_missing_{}", std::process::id());

  for attempt in 1..=2 {
    let found = dp
      .find_topic(&name, Duration::from_millis(10))
      .unwrap_or_else(|e| panic!("missing-topic lookup {attempt} failed: {e}"));
    assert!(found.is_none(), "the topic should not exist");
  }
}

#[test]
fn find_topic_can_be_reused_after_success() {
  let dp = DomainParticipant::new(scratch_domain_id(190)).expect("failed to create participant");
  let name = format!("find_topic_existing_{}", std::process::id());
  let qos = QosPolicyBuilder::new()
    .reliability(policy::Reliability::BestEffort)
    .durability(policy::Durability::Volatile)
    .history(policy::History::KeepLast { depth: 1 })
    .build();
  let topic = dp
    .create_topic(name.clone(), "Sample".to_string(), &qos, TopicKind::NoKey)
    .expect("failed to create Topic");
  let publisher = dp
    .create_publisher(&qos)
    .expect("failed to create Publisher");
  // A live writer registers the topic in DiscoveryDB and keeps it available.
  let _writer = publisher
    .create_datawriter_no_key_cdr::<Sample>(&topic, None)
    .expect("failed to create DataWriter");

  for attempt in 1..=2 {
    let found = dp
      .find_topic(&name, Duration::from_millis(100))
      .unwrap_or_else(|e| panic!("existing-topic lookup {attempt} failed: {e}"));
    assert!(found.is_some(), "the topic should remain discoverable");
  }
}
