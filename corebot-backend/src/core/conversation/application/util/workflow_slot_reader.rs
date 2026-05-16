use chrono::{NaiveDate, NaiveTime};

use crate::core::conversation::domain::model::conversation::Conversation;
use crate::core::conversation::domain::model::slot::{SlotDataValue, SlotName};

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationCreateSlots {
    pub name: String,
    pub date: Option<NaiveDate>,
    pub time: Option<NaiveTime>,
    pub people_count: u32,
}

impl ReservationCreateSlots {
    pub fn from_conversation(conversation: &Conversation) -> Self {
        Self {
            name: text_slot(conversation, SlotName::Name),
            date: date_slot(conversation, SlotName::Date),
            time: time_slot(conversation, SlotName::Time),
            people_count: number_slot(conversation, SlotName::People),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationCancelSlots {
    pub reference: String,
    pub name: Option<String>,
    pub date: Option<NaiveDate>,
}

impl ReservationCancelSlots {
    pub fn from_conversation(conversation: &Conversation) -> Self {
        let name = text_slot(conversation, SlotName::Name);
        Self {
            reference: text_slot(conversation, SlotName::Reference),
            name: if name.is_empty() { None } else { Some(name) },
            date: date_slot(conversation, SlotName::Date),
        }
    }
}

fn slot_value(conversation: &Conversation, slot: SlotName) -> Option<&SlotDataValue> {
    conversation
        .active_workflow()
        .and_then(|workflow| workflow.slot_value(slot))
}

fn text_slot(conversation: &Conversation, slot: SlotName) -> String {
    match slot_value(conversation, slot) {
        Some(SlotDataValue::Text(value)) => value.clone(),
        Some(SlotDataValue::Number(value)) => value.to_string(),
        Some(SlotDataValue::Boolean(value)) => value.to_string(),
        _ => String::new(),
    }
}

fn date_slot(conversation: &Conversation, slot: SlotName) -> Option<NaiveDate> {
    match slot_value(conversation, slot) {
        Some(SlotDataValue::Date(value)) => Some(*value),
        _ => None,
    }
}

fn time_slot(conversation: &Conversation, slot: SlotName) -> Option<NaiveTime> {
    match slot_value(conversation, slot) {
        Some(SlotDataValue::Time(value)) => Some(*value),
        _ => None,
    }
}

fn number_slot(conversation: &Conversation, slot: SlotName) -> u32 {
    match slot_value(conversation, slot) {
        Some(SlotDataValue::Number(value)) => *value,
        _ => 0,
    }
}
