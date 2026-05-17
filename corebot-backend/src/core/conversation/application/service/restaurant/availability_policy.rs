use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use std::collections::HashMap;

use crate::core::conversation::domain::restaurant::model::{OpeningHours, Reservation, TableType};

pub(super) fn is_open_at(
    opening_hours: &[OpeningHours],
    date: NaiveDate,
    time: NaiveTime,
    slot_minutes: u32,
) -> bool {
    let Some(opening) = opening_hours
        .iter()
        .find(|entry| entry.day_of_week == date.weekday())
    else {
        return false;
    };
    if opening.is_closed || time < opening.opens_at {
        return false;
    }
    let slot_end_secs = time.num_seconds_from_midnight() as i64 + (slot_minutes as i64 * 60);
    slot_end_secs <= opening.closes_at.num_seconds_from_midnight() as i64
}

pub(super) fn can_seat(
    tables: &[TableType],
    reservations: &[Reservation],
    date: NaiveDate,
    time: NaiveTime,
    people: u32,
    slot_minutes: u32,
) -> bool {
    let booked = booked_tables(tables, reservations, date, time, slot_minutes);
    let mut available = tables
        .iter()
        .map(|table| {
            let used = booked.get(&table.capacity).copied().unwrap_or(0);
            (table.capacity, table.count.saturating_sub(used))
        })
        .filter(|(_, count)| *count > 0)
        .collect::<Vec<_>>();
    available.sort_by(|left, right| right.0.cmp(&left.0));

    let mut remaining = people;
    for (capacity, count) in available {
        if remaining == 0 {
            break;
        }
        let needed = remaining.div_ceil(capacity);
        let used = needed.min(count);
        remaining = remaining.saturating_sub(used * capacity);
    }
    remaining == 0
}

fn booked_tables(
    tables: &[TableType],
    reservations: &[Reservation],
    date: NaiveDate,
    time: NaiveTime,
    slot_minutes: u32,
) -> HashMap<u32, u32> {
    let slot_secs = (slot_minutes * 60) as i64;
    let req_start = NaiveDateTime::new(date, time);
    let req_end = req_start + Duration::seconds(slot_secs);
    let mut used = tables
        .iter()
        .map(|table| (table.capacity, 0_u32))
        .collect::<HashMap<_, _>>();
    let mut tables_asc = tables
        .iter()
        .map(|table| (table.capacity, table.count))
        .collect::<Vec<_>>();
    tables_asc.sort_by_key(|(capacity, _)| *capacity);

    for reservation in reservations
        .iter()
        .filter(|reservation| reservation.date == date)
    {
        let res_start = NaiveDateTime::new(reservation.date, reservation.time);
        let res_end = res_start + Duration::seconds(slot_secs);
        if req_start >= res_end || res_start >= req_end {
            continue;
        }
        let mut remaining = reservation.people_count;
        let mut tables_desc = tables_asc.clone();
        tables_desc.sort_by(|left, right| right.0.cmp(&left.0));
        for (capacity, _) in &tables_desc {
            if remaining == 0 {
                break;
            }
            let available = tables_asc
                .iter()
                .find(|(candidate, _)| candidate == capacity)
                .map(|(_, count)| count.saturating_sub(*used.get(capacity).unwrap_or(&0)))
                .unwrap_or(0);
            if available == 0 {
                continue;
            }
            let needed = remaining.div_ceil(*capacity).min(available);
            *used.entry(*capacity).or_insert(0) += needed;
            remaining = remaining.saturating_sub(needed * capacity);
        }
    }
    used
}
