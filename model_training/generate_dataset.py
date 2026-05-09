"""Generate deterministic EN/ID restaurant NLU datasets."""

from __future__ import annotations

import random
from typing import Any

from nlu_training.config import load_config
from nlu_training.schema import load_jsonl, validate_examples, write_jsonl


SEED = 42

EN_VALUES = {
    "person": ["Jean Martin", "Maya Chen", "Alex Carter", "Priya Singh", "Noah Davis"],
    "date": ["today", "tomorrow", "Friday", "June 12", "next Monday"],
    "time": ["7pm", "8:30pm", "19:00", "noon", "6 pm"],
    "people_count": ["2 people", "4 people", "6 people", "3 people", "5 people"],
    "menu_item": ["pizza", "salad", "chocolate cake", "fried rice", "vegetarian pasta"],
    "price_item": ["set menu", "wine list", "dessert menu", "lunch special", "kids menu"],
    "location": ["downtown", "near the station", "main branch", "terrace", "private room"],
    "phone": ["+33123456789", "01 23 45 67 89", "+33111222333"],
    "email": ["events@example.com", "maya@example.com", "hello@example.com"],
    "dietary_requirement": ["vegan", "halal", "gluten-free"],
    "allergen": ["tomatoes", "gluten", "eggs"],
    "facility": ["baby seat", "parking", "smoking area"],
    "payment_method": ["credit card", "cash", "Apple Pay"],
    "reservation_reference": ["ABC123", "ZX90", "REF202"],
}

ID_VALUES = {
    "person": ["Budi Santoso", "Siti Aminah", "Maya Chen", "Agus Wijaya", "Dewi Lestari"],
    "date": ["hari ini", "besok", "Jumat", "12 Juni", "Senin depan"],
    "time": ["jam 7 malam", "20.30", "pukul 19:00", "siang", "jam 6 sore"],
    "people_count": ["2 orang", "4 orang", "6 orang", "3 orang", "5 orang"],
    "menu_item": ["nasi goreng", "sate ayam", "salad", "pizza", "kue cokelat"],
    "price_item": ["menu paket", "daftar wine", "menu pencuci mulut", "promo makan siang", "menu anak"],
    "location": ["pusat kota", "dekat stasiun", "cabang utama", "teras", "ruang privat"],
    "phone": ["+628123456789", "021 555 123", "+628111222333"],
    "email": ["acara@example.com", "siti@example.com", "halo@example.com"],
    "dietary_requirement": ["vegan", "halal", "bebas gluten"],
    "allergen": ["tomat", "gluten", "telur"],
    "facility": ["kursi bayi", "parkiran", "area merokok"],
    "payment_method": ["kartu kredit", "tunai", "QRIS"],
    "reservation_reference": ["ABC123", "ZX90", "REF202"],
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


def rows_from_templates(
    templates: list[tuple[str, str, list[tuple[str, str]], str | None]],
    lang: str,
) -> list[dict[str, Any]]:
    return [row(text, intent, lang, entities, task) for intent, text, entities, task in templates]


def build_english_rows() -> list[dict[str, Any]]:
    values = EN_VALUES
    rows: list[dict[str, Any]] = []

    for person, date, time, people, reference in zip(
        values["person"],
        values["date"],
        values["time"],
        values["people_count"],
        values["reservation_reference"],
    ):
        rows.extend(
            [
                row(
                    f"I want to book a table for {people} on {date} at {time} under {person}",
                    "reservation_create",
                    "en",
                    [("people_count", people), ("date", date), ("time", time), ("person", person)],
                ),
                row(
                    f"{person}",
                    "reservation_create",
                    "en",
                    [("person", person)],
                    task="WF_RESERVATION_CREATE",
                ),
                row(
                    f"for {people} on {date} at {time}",
                    "reservation_create",
                    "en",
                    [("people_count", people), ("date", date), ("time", time)],
                    task="WF_RESERVATION_CREATE",
                ),
                row(
                    f"I want to cancel my reservation under {person} on {date} at {time}",
                    "reservation_cancel",
                    "en",
                    [("person", person), ("date", date), ("time", time)],
                ),
                row(
                    f"reference {reference}",
                    "reservation_cancel",
                    "en",
                    [("reservation_reference", reference)],
                    task="WF_RESERVATION_CANCEL",
                ),
                row(
                    f"It is under {person} on {date}",
                    "reservation_cancel",
                    "en",
                    [("person", person), ("date", date)],
                    task="WF_RESERVATION_CANCEL",
                ),
                row(
                    "Please cancel this booking flow",
                    "cancel",
                    "en",
                    task="WF_RESERVATION_CREATE",
                ),
                row(
                    "Stop this cancellation flow",
                    "cancel",
                    "en",
                    task="WF_RESERVATION_CANCEL",
                ),
                row(f"Do I have a reservation with reference {reference}?", "check_reservation", "en", [("reservation_reference", reference)]),
            ]
        )

    templates = [
        ("ask_opening_hours", "What time are you open?", [], None),
        ("ask_opening_hours", "Are you open tomorrow at 8?", [("date", "tomorrow"), ("time", "8")], None),
        ("ask_opening_hours", "What time do you close on Friday?", [("date", "Friday")], None),
        ("ask_menu_general", "Can I see the menu?", [], None),
        ("ask_menu_general", "Can I have the dessert menu?", [("price_item", values["price_item"][2])], None),
        ("ask_menu_general", "Please send me your drink list", [], None),
        ("ask_menu_dietary", f"Do you have {values['dietary_requirement'][0]} dishes?", [("dietary_requirement", values["dietary_requirement"][0])], None),
        ("ask_menu_dietary", f"Which dishes are {values['dietary_requirement'][2]}?", [("dietary_requirement", values["dietary_requirement"][2])], None),
        ("ask_menu_dietary", f"Do you have {values['dietary_requirement'][1]} options for kids?", [("dietary_requirement", values["dietary_requirement"][1])], None),
        ("ask_menu_item_details", f"Which dish contains {values['allergen'][0]}?", [("allergen", values["allergen"][0])], None),
        ("ask_menu_item_details", f"Does the {values['menu_item'][0]} contain {values['allergen'][1]}?", [("menu_item", values["menu_item"][0]), ("allergen", values["allergen"][1])], None),
        ("ask_menu_item_details", f"What is in the {values['menu_item'][2]}?", [("menu_item", values["menu_item"][2])], None),
        ("ask_location", "Can you share your address?", [], None),
        ("ask_location", f"Are you near {values['location'][1]}?", [("location", values["location"][1])], None),
        ("ask_location", f"Is your restaurant close to {values['location'][0]}?", [("location", values["location"][0])], None),
        ("ask_contact", "Could you provide your telephone number?", [], None),
        ("ask_contact", f"Can you contact me at {values['phone'][0]}?", [("phone", values["phone"][0])], None),
        ("ask_contact", f"My email is {values['email'][0]}", [("email", values["email"][0])], None),
        ("ask_payment_methods", f"Can I pay by {values['payment_method'][0]}?", [("payment_method", values["payment_method"][0])], None),
        ("ask_payment_methods", f"Do you accept {values['payment_method'][1]}?", [("payment_method", values["payment_method"][1])], None),
        ("ask_payment_methods", "Can I split the bill?", [], None),
        ("ask_price", f"How much is the {values['price_item'][0]}?", [("price_item", values["price_item"][0])], None),
        ("ask_price", f"What is the price of the {values['price_item'][3]}?", [("price_item", values["price_item"][3])], None),
        ("ask_price", "Is there a service charge?", [], None),
        ("ask_takeaway_delivery", "I would like a delivery order.", [], None),
        ("ask_takeaway_delivery", "Do you offer takeout?", [], None),
        ("ask_takeaway_delivery", "Can I get this meal delivered?", [], None),
        ("ask_event", "Do you host birthday parties?", [], None),
        ("ask_event", "Can I organize a business event there?", [], None),
        ("ask_event", f"Do you have a {values['location'][4]} for an event?", [("location", values["location"][4])], None),
        ("ask_facilities", f"Do you have a {values['facility'][0]}?", [("facility", values["facility"][0])], None),
        ("ask_facilities", f"Is there {values['facility'][1]} near your restaurant?", [("facility", values["facility"][1])], None),
        ("ask_facilities", f"Do you have a {values['facility'][2]}?", [("facility", values["facility"][2])], None),
        ("ask_accessibility", "Is the restaurant wheelchair accessible?", [], None),
        ("ask_accessibility", "Do you have disabled-friendly access?", [], None),
        ("ask_accessibility", "Is there easy access for a stroller and wheelchair?", [], None),
        ("ask_entertainment", "Do you have live music?", [], None),
        ("ask_entertainment", "Is there karaoke tonight?", [], None),
        ("ask_entertainment", "Do you host concerts on weekends?", [], None),
        ("greeting", "Hello", [], None),
        ("greeting", "Hi there", [], None),
        ("greeting", "Good evening", [], None),
        ("thanks", "Thank you", [], None),
        ("thanks", "Thanks a lot", [], None),
        ("thanks", "I appreciate it", [], None),
        ("goodbye", "Goodbye", [], None),
        ("goodbye", "Bye", [], None),
        ("goodbye", "See you later", [], None),
        ("affirmative", "Yes", [], "WF_CHOICE"),
        ("affirmative", "Yes please", [], "WF_CHOICE"),
        ("affirmative", "That is correct", [], "WF_CHOICE"),
        ("negative", "No", [], "WF_CHOICE"),
        ("negative", "No thanks", [], "WF_CHOICE"),
        ("negative", "That is not right", [], "WF_CHOICE"),
        ("unknown", "Can you book me a flight to Bali?", [], None),
        ("unknown", "Maybe purple clouds can reserve me a seat", [], None),
        ("unknown", "Maybe later", [], "WF_CHOICE"),
        ("cancel", "Finally, I want to cancel this reservation", [], "WF_RESERVATION_CREATE"),
    ]
    rows.extend(rows_from_templates(templates, "en"))
    return rows


def build_indonesian_rows() -> list[dict[str, Any]]:
    values = ID_VALUES
    rows: list[dict[str, Any]] = []

    for person, date, time, people, reference in zip(
        values["person"],
        values["date"],
        values["time"],
        values["people_count"],
        values["reservation_reference"],
    ):
        rows.extend(
            [
                row(
                    f"Saya mau pesan meja untuk {people} pada {date} jam {time} atas nama {person}",
                    "reservation_create",
                    "id",
                    [("people_count", people), ("date", date), ("time", time), ("person", person)],
                ),
                row(
                    f"{person}",
                    "reservation_create",
                    "id",
                    [("person", person)],
                    task="WF_RESERVATION_CREATE",
                ),
                row(
                    f"untuk {people} pada {date} jam {time}",
                    "reservation_create",
                    "id",
                    [("people_count", people), ("date", date), ("time", time)],
                    task="WF_RESERVATION_CREATE",
                ),
                row(
                    f"Saya mau membatalkan reservasi atas nama {person} pada {date} jam {time}",
                    "reservation_cancel",
                    "id",
                    [("person", person), ("date", date), ("time", time)],
                ),
                row(
                    f"referensi {reference}",
                    "reservation_cancel",
                    "id",
                    [("reservation_reference", reference)],
                    task="WF_RESERVATION_CANCEL",
                ),
                row(
                    f"atas nama {person} pada {date}",
                    "reservation_cancel",
                    "id",
                    [("person", person), ("date", date)],
                    task="WF_RESERVATION_CANCEL",
                ),
                row(
                    "Tolong batalkan alur pemesanan ini",
                    "cancel",
                    "id",
                    task="WF_RESERVATION_CREATE",
                ),
                row(
                    "Hentikan alur pembatalan ini",
                    "cancel",
                    "id",
                    task="WF_RESERVATION_CANCEL",
                ),
                row(
                    f"Apakah saya punya reservasi dengan referensi {reference}?",
                    "check_reservation",
                    "id",
                    [("reservation_reference", reference)],
                ),
            ]
        )

    templates = [
        ("ask_opening_hours", "Jam berapa buka?", [], None),
        ("ask_opening_hours", "Apakah buka besok jam 8?", [("date", "besok"), ("time", "8")], None),
        ("ask_opening_hours", "Jam berapa tutup hari Jumat?", [("date", "Jumat")], None),
        ("ask_menu_general", "Boleh lihat menunya?", [], None),
        ("ask_menu_general", f"Boleh minta {values['price_item'][2]}?", [("price_item", values["price_item"][2])], None),
        ("ask_menu_general", "Tolong kirim daftar minumannya", [], None),
        ("ask_menu_dietary", f"Ada hidangan {values['dietary_requirement'][0]}?", [("dietary_requirement", values["dietary_requirement"][0])], None),
        ("ask_menu_dietary", f"Menu mana yang {values['dietary_requirement'][2]}?", [("dietary_requirement", values["dietary_requirement"][2])], None),
        ("ask_menu_dietary", f"Ada pilihan {values['dietary_requirement'][1]} untuk anak?", [("dietary_requirement", values["dietary_requirement"][1])], None),
        ("ask_menu_item_details", f"Hidangan mana yang mengandung {values['allergen'][0]}?", [("allergen", values["allergen"][0])], None),
        ("ask_menu_item_details", f"Apakah {values['menu_item'][3]} mengandung {values['allergen'][1]}?", [("menu_item", values["menu_item"][3]), ("allergen", values["allergen"][1])], None),
        ("ask_menu_item_details", f"Apa isi {values['menu_item'][4]}?", [("menu_item", values["menu_item"][4])], None),
        ("ask_location", "Bisa kirim alamatnya?", [], None),
        ("ask_location", f"Apakah restoran dekat {values['location'][1]}?", [("location", values["location"][1])], None),
        ("ask_location", f"Apakah lokasinya di {values['location'][0]}?", [("location", values["location"][0])], None),
        ("ask_contact", "Boleh kasih nomor teleponnya?", [], None),
        ("ask_contact", f"Bisa hubungi saya di {values['phone'][0]}?", [("phone", values["phone"][0])], None),
        ("ask_contact", f"Email saya {values['email'][0]}", [("email", values["email"][0])], None),
        ("ask_payment_methods", f"Bisa bayar pakai {values['payment_method'][0]}?", [("payment_method", values["payment_method"][0])], None),
        ("ask_payment_methods", f"Apakah menerima {values['payment_method'][1]}?", [("payment_method", values["payment_method"][1])], None),
        ("ask_payment_methods", "Bisa pisah tagihan?", [], None),
        ("ask_price", f"Berapa harga {values['price_item'][0]}?", [("price_item", values["price_item"][0])], None),
        ("ask_price", f"Berapa harga {values['price_item'][3]}?", [("price_item", values["price_item"][3])], None),
        ("ask_price", "Ada biaya layanan?", [], None),
        ("ask_takeaway_delivery", "Saya mau pesan delivery.", [], None),
        ("ask_takeaway_delivery", "Apakah ada takeout?", [], None),
        ("ask_takeaway_delivery", "Bisa dikirim ke rumah?", [], None),
        ("ask_event", "Apakah bisa untuk pesta ulang tahun?", [], None),
        ("ask_event", "Bisa buat acara kantor?", [], None),
        ("ask_event", f"Ada {values['location'][4]} untuk acara?", [("location", values["location"][4])], None),
        ("ask_facilities", f"Ada {values['facility'][0]}?", [("facility", values["facility"][0])], None),
        ("ask_facilities", f"Ada {values['facility'][1]} dekat restoran?", [("facility", values["facility"][1])], None),
        ("ask_facilities", f"Ada {values['facility'][2]}?", [("facility", values["facility"][2])], None),
        ("ask_accessibility", "Apakah restoran ramah kursi roda?", [], None),
        ("ask_accessibility", "Apakah aksesnya ramah disabilitas?", [], None),
        ("ask_accessibility", "Apakah mudah diakses stroller dan kursi roda?", [], None),
        ("ask_entertainment", "Ada live music?", [], None),
        ("ask_entertainment", "Ada karaoke malam ini?", [], None),
        ("ask_entertainment", "Ada konser saat akhir pekan?", [], None),
        ("greeting", "Halo", [], None),
        ("greeting", "Hai", [], None),
        ("greeting", "Selamat malam", [], None),
        ("thanks", "Terima kasih", [], None),
        ("thanks", "Makasih banyak", [], None),
        ("thanks", "Saya menghargainya", [], None),
        ("goodbye", "Sampai jumpa", [], None),
        ("goodbye", "Dadah", [], None),
        ("goodbye", "Sampai nanti", [], None),
        ("affirmative", "Ya", [], "WF_CHOICE"),
        ("affirmative", "Ya silakan", [], "WF_CHOICE"),
        ("affirmative", "Itu benar", [], "WF_CHOICE"),
        ("negative", "Tidak", [], "WF_CHOICE"),
        ("negative", "Tidak terima kasih", [], "WF_CHOICE"),
        ("negative", "Itu tidak benar", [], "WF_CHOICE"),
        ("unknown", "Bisa pesankan tiket pesawat ke Bali?", [], None),
        ("unknown", "Mungkin awan ungu bisa pilihkan meja", [], None),
        ("unknown", "Mungkin nanti", [], "WF_CHOICE"),
        ("cancel", "Akhirnya saya mau membatalkan reservasi ini", [], "WF_RESERVATION_CREATE"),
    ]
    rows.extend(rows_from_templates(templates, "id"))
    return rows


def split_rows(rows: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    rng = random.Random(SEED)
    grouped: dict[str, list[dict[str, Any]]] = {}
    for item in rows:
        grouped.setdefault(f"{item['lang']}:{item['intent']}:{item.get('task', '-')}", []).append(item)

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
