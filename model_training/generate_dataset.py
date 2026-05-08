"""Generate deterministic EN/ID restaurant NLU datasets."""

from __future__ import annotations

import random
from pathlib import Path
from typing import Any

from nlu_training.config import load_config
from nlu_training.schema import TrainingExample, load_jsonl, validate_examples, write_jsonl


SEED = 42

EN_VALUES = {
    "person": ["Jean Martin", "Maya Chen", "Alex Carter", "Priya Singh", "Noah Davis"],
    "date": ["today", "tomorrow", "Friday", "June 12", "next Monday"],
    "time": ["7pm", "8:30pm", "19:00", "noon", "6 pm"],
    "people_count": ["2", "4 people", "six people", "3", "5 people"],
    "menu_item": ["pizza", "salad", "chocolate cake", "fried rice", "vegetarian pasta"],
    "price_item": ["set menu", "wine list", "dessert", "lunch special", "kids menu"],
    "location": ["downtown", "near the station", "main branch", "terrace", "private room"],
    "phone": ["+33123456789", "01 23 45 67 89"],
    "email": ["events@example.com", "maya@example.com"],
}

ID_VALUES = {
    "person": ["Budi Santoso", "Siti Aminah", "Maya Chen", "Agus Wijaya", "Dewi Lestari"],
    "date": ["hari ini", "besok", "Jumat", "12 Juni", "Senin depan"],
    "time": ["jam 7 malam", "20.30", "pukul 19:00", "siang", "jam 6 sore"],
    "people_count": ["2 orang", "empat orang", "6 orang", "3 orang", "5 orang"],
    "menu_item": ["nasi goreng", "sate ayam", "salad", "pizza", "kue cokelat"],
    "price_item": ["menu paket", "daftar wine", "hidangan penutup", "promo makan siang", "menu anak"],
    "location": ["pusat kota", "dekat stasiun", "cabang utama", "teras", "ruang privat"],
    "phone": ["+628123456789", "021 555 123"],
    "email": ["acara@example.com", "siti@example.com"],
}


def span_entities(text: str, typed_values: list[tuple[str, str]]) -> list[dict[str, Any]]:
    entities: list[dict[str, Any]] = []
    search_from = 0
    for entity_type, value in typed_values:
        start = text.find(value, search_from)
        if start < 0:
            start = text.find(value)
        if start < 0:
            raise ValueError(f"Could not find {value!r} in {text!r}")
        end = start + len(value)
        entities.append({"start": start, "end": end, "type": entity_type})
        search_from = end
    return entities


def row(
    text: str,
    intent: str,
    lang: str,
    entities: list[tuple[str, str]] | None = None,
    task: str | None = None,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "text": text,
        "lang": lang,
        "domain": "restaurant",
        "intent": intent,
        "entities": span_entities(text, entities or []),
    }
    if task is not None:
        payload["task"] = task
    return payload


def build_english_rows() -> list[dict[str, Any]]:
    values = EN_VALUES
    rows: list[dict[str, Any]] = []
    for person, date, time, people in zip(values["person"], values["date"], values["time"], values["people_count"]):
        rows.extend(
            [
                row(
                    f"I want to book a table for {people} {date} at {time} under {person}",
                    "book",
                    "en",
                    [("people_count", people), ("date", date), ("time", time), ("person", person)],
                ),
                row(
                    f"{person} for {people} {date} at {time}",
                    "provide_info",
                    "en",
                    [("person", person), ("people_count", people), ("date", date), ("time", time)],
                    task="WF_BOOK",
                ),
                row(
                    f"{people} {date} {time}",
                    "provide_info",
                    "en",
                    [("people_count", people), ("date", date), ("time", time)],
                    task="WF_BOOK",
                ),
                row(
                    f"Cancel the booking for {person} on {date} at {time}",
                    "cancel",
                    "en",
                    [("person", person), ("date", date), ("time", time)],
                ),
                row(
                    f"It is under {person} for {date}",
                    "provide_info",
                    "en",
                    [("person", person), ("date", date)],
                    task="WF_CANCEL",
                ),
                row(f"Do I have a reservation for {date}?", "reservation_status", "en", [("date", date)]),
            ]
        )

    intent_templates = [
        ("ask_menu", "Can I see the menu?", []),
        ("ask_menu", f"Do you serve {values['menu_item'][0]}?", [("menu_item", values["menu_item"][0])]),
        ("ask_hours", "What time are you open?", []),
        ("ask_location", f"Where is the {values['location'][2]}?", [("location", values["location"][2])]),
        ("ask_contact", "What phone number should I call?", []),
        ("ask_contact", f"Can you contact me at {values['phone'][0]}?", [("phone", values["phone"][0])]),
        ("ask_price", f"How much is the {values['price_item'][0]}?", [("price_item", values["price_item"][0])]),
        ("ask_availability", f"Is the {values['location'][3]} available tomorrow?", [("location", values["location"][3]), ("date", "tomorrow")]),
        ("ask_payment", "Can I pay by credit card?", []),
        ("help", "What can you help me with?", []),
        ("complain", "My table was not ready and I am unhappy", []),
        ("out_of_scope", "Can you book me a flight to Bali?", []),
        ("greeting", "Hello", []),
        ("thanks", "Thank you", []),
        ("goodbye", "Goodbye", []),
        ("affirmative", "Yes, that is correct", []),
        ("negative", "No, not now", []),
    ]
    rows.extend(row(text, intent, "en", entities) for intent, text, entities in intent_templates)
    extra_templates = [
        ("ask_menu", "What dishes are on the menu?", []),
        ("ask_menu", f"Is {values['menu_item'][2]} available?", [("menu_item", values["menu_item"][2])]),
        ("ask_hours", "Are you open tonight?", []),
        ("ask_hours", "What are your opening hours on Friday?", [("date", "Friday")]),
        ("ask_location", "What is your address?", []),
        ("ask_location", f"Can I sit in the {values['location'][3]}?", [("location", values["location"][3])]),
        ("ask_contact", f"My email is {values['email'][0]}", [("email", values["email"][0])]),
        ("ask_contact", "How can I reach the restaurant?", []),
        ("ask_price", f"What is the price of {values['price_item'][3]}?", [("price_item", values["price_item"][3])]),
        ("ask_price", "Is there a service charge?", []),
        ("ask_availability", f"Do you have room for {values['people_count'][1]} at {values['time'][1]}?", [("people_count", values["people_count"][1]), ("time", values["time"][1])]),
        ("ask_availability", f"Any tables free {values['date'][2]}?", [("date", values["date"][2])]),
        ("ask_payment", "Do you accept cash?", []),
        ("ask_payment", "Can I split the bill?", []),
        ("help", "I need help with a reservation", []),
        ("help", "What can this chatbot do?", []),
        ("complain", "I want to complain about slow service", []),
        ("complain", "The food was cold", []),
        ("out_of_scope", "What is the weather tomorrow?", [("date", "tomorrow")]),
        ("out_of_scope", "Can you order a taxi?", []),
        ("greeting", "Hi there", []),
        ("greeting", "Good evening", []),
        ("thanks", "Thanks a lot", []),
        ("thanks", "I appreciate it", []),
        ("goodbye", "Bye", []),
        ("goodbye", "See you later", []),
        ("affirmative", "Yes please", []),
        ("affirmative", "That works", []),
        ("negative", "No thanks", []),
        ("negative", "That is not right", []),
    ]
    rows.extend(row(text, intent, "en", entities) for intent, text, entities in extra_templates)
    return rows


def build_indonesian_rows() -> list[dict[str, Any]]:
    values = ID_VALUES
    rows: list[dict[str, Any]] = []
    for person, date, time, people in zip(values["person"], values["date"], values["time"], values["people_count"]):
        rows.extend(
            [
                row(
                    f"Saya mau pesan meja untuk {people} {date} pada {time} atas nama {person}",
                    "book",
                    "id",
                    [("people_count", people), ("date", date), ("time", time), ("person", person)],
                ),
                row(
                    f"{person} untuk {people} {date} pada {time}",
                    "provide_info",
                    "id",
                    [("person", person), ("people_count", people), ("date", date), ("time", time)],
                    task="WF_BOOK",
                ),
                row(
                    f"{people} {date} {time}",
                    "provide_info",
                    "id",
                    [("people_count", people), ("date", date), ("time", time)],
                    task="WF_BOOK",
                ),
                row(
                    f"Batalkan reservasi atas nama {person} pada {date} jam {time}",
                    "cancel",
                    "id",
                    [("person", person), ("date", date), ("time", time)],
                ),
                row(
                    f"Atas nama {person} untuk {date}",
                    "provide_info",
                    "id",
                    [("person", person), ("date", date)],
                    task="WF_CANCEL",
                ),
                row(f"Apakah saya punya reservasi untuk {date}?", "reservation_status", "id", [("date", date)]),
            ]
        )

    intent_templates = [
        ("ask_menu", "Boleh lihat menunya?", []),
        ("ask_menu", f"Ada {values['menu_item'][0]}?", [("menu_item", values["menu_item"][0])]),
        ("ask_hours", "Jam berapa buka?", []),
        ("ask_location", f"Di mana {values['location'][2]}?", [("location", values["location"][2])]),
        ("ask_contact", "Nomor teleponnya berapa?", []),
        ("ask_contact", f"Bisa hubungi saya di {values['phone'][0]}?", [("phone", values["phone"][0])]),
        ("ask_price", f"Berapa harga {values['price_item'][0]}?", [("price_item", values["price_item"][0])]),
        ("ask_availability", f"Apakah {values['location'][3]} tersedia besok?", [("location", values["location"][3]), ("date", "besok")]),
        ("ask_payment", "Bisa bayar dengan kartu kredit?", []),
        ("help", "Kamu bisa bantu apa?", []),
        ("complain", "Meja saya belum siap dan saya kecewa", []),
        ("out_of_scope", "Tolong pesankan tiket pesawat ke Bali", []),
        ("greeting", "Halo", []),
        ("thanks", "Terima kasih", []),
        ("goodbye", "Sampai jumpa", []),
        ("affirmative", "Ya, benar", []),
        ("negative", "Tidak, nanti saja", []),
    ]
    rows.extend(row(text, intent, "id", entities) for intent, text, entities in intent_templates)
    extra_templates = [
        ("ask_menu", "Ada hidangan apa di menu?", []),
        ("ask_menu", f"Apakah {values['menu_item'][2]} tersedia?", [("menu_item", values["menu_item"][2])]),
        ("ask_hours", "Apakah buka malam ini?", []),
        ("ask_hours", "Jam buka hari Jumat?", [("date", "Jumat")]),
        ("ask_location", "Alamatnya di mana?", []),
        ("ask_location", f"Bisa duduk di {values['location'][3]}?", [("location", values["location"][3])]),
        ("ask_contact", f"Email saya {values['email'][0]}", [("email", values["email"][0])]),
        ("ask_contact", "Bagaimana cara menghubungi restoran?", []),
        ("ask_price", f"Berapa harga {values['price_item'][3]}?", [("price_item", values["price_item"][3])]),
        ("ask_price", "Ada biaya layanan?", []),
        ("ask_availability", f"Ada meja untuk {values['people_count'][1]} jam {values['time'][1]}?", [("people_count", values["people_count"][1]), ("time", values["time"][1])]),
        ("ask_availability", f"Ada meja kosong {values['date'][2]}?", [("date", values["date"][2])]),
        ("ask_payment", "Bisa bayar tunai?", []),
        ("ask_payment", "Bisa pisah tagihan?", []),
        ("help", "Saya butuh bantuan reservasi", []),
        ("help", "Chatbot ini bisa apa?", []),
        ("complain", "Saya ingin komplain tentang layanan lambat", []),
        ("complain", "Makanannya dingin", []),
        ("out_of_scope", "Bagaimana cuaca besok?", [("date", "besok")]),
        ("out_of_scope", "Bisa pesankan taksi?", []),
        ("greeting", "Hai", []),
        ("greeting", "Selamat malam", []),
        ("thanks", "Makasih banyak", []),
        ("thanks", "Saya menghargainya", []),
        ("goodbye", "Dadah", []),
        ("goodbye", "Sampai nanti", []),
        ("affirmative", "Ya silakan", []),
        ("affirmative", "Itu cocok", []),
        ("negative", "Tidak terima kasih", []),
        ("negative", "Itu tidak benar", []),
    ]
    rows.extend(row(text, intent, "id", entities) for intent, text, entities in extra_templates)
    return rows


def split_rows(rows: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    rng = random.Random(SEED)
    grouped: dict[str, list[dict[str, Any]]] = {}
    for item in rows:
        grouped.setdefault(f"{item['lang']}:{item['intent']}", []).append(item)

    train: list[dict[str, Any]] = []
    validation: list[dict[str, Any]] = []
    eval_rows: list[dict[str, Any]] = []
    for group_rows in grouped.values():
        rng.shuffle(group_rows)
        if len(group_rows) >= 3:
            validation.append(group_rows[0])
            eval_rows.append(group_rows[1])
            train.extend(group_rows[2:])
        elif len(group_rows) == 2:
            validation.append(group_rows[0])
            train.append(group_rows[1])
        else:
            train.extend(group_rows)
    rng.shuffle(train)
    rng.shuffle(validation)
    rng.shuffle(eval_rows)
    return train, validation, eval_rows


def main() -> None:
    config = load_config()
    rows = build_english_rows() + build_indonesian_rows()
    train, validation, eval_rows = split_rows(rows)

    data_config = config["data"]
    write_jsonl(data_config["train"], train)
    write_jsonl(data_config["validation"], validation)
    write_jsonl(data_config["eval"], eval_rows)

    for path in (data_config["train"], data_config["validation"], data_config["eval"]):
        examples = load_jsonl(path)
        validate_examples(examples, config)

    print(f"Generated {len(train)} train, {len(validation)} validation, {len(eval_rows)} eval examples.")


if __name__ == "__main__":
    main()
