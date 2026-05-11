"""Generate deterministic EN/ID restaurant NLU datasets."""

from __future__ import annotations

import random
import re
from collections.abc import Callable
from typing import Any

from nlu_training.config import load_config
from nlu_training.schema import load_jsonl, validate_examples, write_jsonl


SEED = 42
TARGET_ROWS_PER_LANGUAGE = 2500


VALUES = {
    "en": {
        "person": [
            "Jean Martin",
            "Maya Chen",
            "Alex Carter",
            "Priya Singh",
            "Noah Davis",
            "Alice Brown",
            "Sam Wilson",
            "Nina Patel",
            "Omar Khan",
            "Lena Smith",
        ],
        "date": [
            "today",
            "tomorrow",
            "Friday",
            "next Monday",
            "on July 8",
            "next Tuesday",
            "on August 23 2026",
            "2026-08-23",
            "23/08/2026",
            "June 12",
        ],
        "time": [
            "7pm",
            "8:30pm",
            "19:00",
            "noon",
            "6 pm",
            "7:30 pm",
            "midday",
            "20:30",
            "9 am",
            "18:45",
        ],
        "people_count": [
            "1 person",
            "2 people",
            "3 people",
            "4 people",
            "5 people",
            "6 people",
            "8 people",
            "10 people",
            "12 people",
            "20 people",
        ],
        "menu_item": [
            "pizza",
            "salad",
            "chocolate cake",
            "fried rice",
            "vegetarian pasta",
            "seafood soup",
            "beef burger",
            "chicken satay",
            "vegan curry",
            "kids pasta",
        ],
        "price_item": [
            "set menu",
            "wine list",
            "dessert menu",
            "lunch special",
            "kids menu",
            "tasting menu",
            "breakfast menu",
            "family menu",
        ],
        "location": [
            "downtown",
            "near the station",
            "main branch",
            "terrace",
            "private room",
            "city center",
            "by the river",
            "near the airport",
        ],
        "phone": ["+33123456789", "01 23 45 67 89", "+33111222333", "+44 20 7946 0958"],
        "email": ["events@example.com", "maya@example.com", "hello@example.com", "booking@example.com"],
        "dietary_requirement": [
            "vegan",
            "vegetarian",
            "halal",
            "gluten-free",
            "lactose-free",
            "dairy-free",
            "nut-free",
            "low-salt",
        ],
        "allergen": [
            "gluten",
            "nuts",
            "peanuts",
            "dairy",
            "eggs",
            "shellfish",
            "soy",
            "sesame",
            "tomatoes",
        ],
        "facility": [
            "baby seat",
            "parking",
            "smoking area",
            "wifi",
            "high chairs",
            "outdoor seating",
            "private room",
            "bike parking",
        ],
        "payment_method": ["credit card", "cash", "Apple Pay", "Google Pay", "Visa", "Mastercard", "contactless"],
        "reservation_reference": [
            "REST-ABC123",
            "REST-ZX90K2",
            "REST-2026A1",
            "REST-7F4K2A",
            "REST-MN45QP",
            "REST-9X8Y7Z",
            "REST-BOOK42",
            "REST-CXL777",
            "REST-A1B2C3",
            "REST-TABLE9",
        ],
        "price_comparator": ["under", "less than", "below", "greater than", "more than", "over"],
        "price_amount": ["20 euros", "$30", "15 euros", "25 dollars", "10", "50 euros", "$45", "35 dollars"],
    },
    "id": {
        "person": [
            "Budi Santoso",
            "Siti Aminah",
            "Maya Chen",
            "Agus Wijaya",
            "Dewi Lestari",
            "Rina Putri",
            "Andi Saputra",
            "Nina Patel",
            "Omar Khan",
            "Lena Smith",
        ],
        "date": [
            "hari ini",
            "besok",
            "Jumat",
            "Senin depan",
            "pada 8 Juli",
            "Selasa depan",
            "pada 23 Agustus 2026",
            "2026-08-23",
            "23/08/2026",
            "12 Juni",
        ],
        "time": [
            "jam 7 malam",
            "20.30",
            "pukul 19:00",
            "siang",
            "jam 6 sore",
            "pukul 7.30 malam",
            "tengah hari",
            "18:45",
            "jam 9 pagi",
            "pukul 20:15",
        ],
        "people_count": [
            "1 orang",
            "2 orang",
            "3 orang",
            "4 orang",
            "5 orang",
            "6 orang",
            "8 orang",
            "10 orang",
            "12 orang",
            "20 orang",
        ],
        "menu_item": [
            "nasi goreng",
            "sate ayam",
            "salad",
            "pizza",
            "kue cokelat",
            "sup seafood",
            "burger sapi",
            "pasta vegetarian",
            "kari vegan",
            "pasta anak",
        ],
        "price_item": [
            "menu paket",
            "daftar wine",
            "menu pencuci mulut",
            "promo makan siang",
            "menu anak",
            "menu degustasi",
            "menu sarapan",
            "menu keluarga",
        ],
        "location": [
            "pusat kota",
            "dekat stasiun",
            "cabang utama",
            "teras",
            "ruang privat",
            "tengah kota",
            "dekat sungai",
            "dekat bandara",
        ],
        "phone": ["+628123456789", "021 555 123", "+628111222333", "0812 3456 7890"],
        "email": ["acara@example.com", "siti@example.com", "halo@example.com", "booking@example.com"],
        "dietary_requirement": [
            "vegan",
            "vegetarian",
            "halal",
            "bebas gluten",
            "bebas laktosa",
            "tanpa susu",
            "tanpa kacang",
            "rendah garam",
        ],
        "allergen": [
            "gluten",
            "kacang",
            "kacang tanah",
            "susu",
            "telur",
            "kerang",
            "kedelai",
            "wijen",
            "tomat",
        ],
        "facility": [
            "kursi bayi",
            "parkiran",
            "area merokok",
            "wifi",
            "kursi tinggi",
            "tempat duduk luar",
            "ruang privat",
            "parkir sepeda",
        ],
        "payment_method": ["kartu kredit", "tunai", "QRIS", "debit", "Visa", "Mastercard", "nirsentuh"],
        "reservation_reference": [
            "REST-ABC123",
            "REST-ZX90K2",
            "REST-2026A1",
            "REST-7F4K2A",
            "REST-MN45QP",
            "REST-9X8Y7Z",
            "REST-BOOK42",
            "REST-CXL777",
            "REST-A1B2C3",
            "REST-TABLE9",
        ],
        "price_comparator": ["di bawah", "kurang dari", "lebih dari", "di atas", "maksimal", "minimal"],
        "price_amount": ["200 ribu", "150000", "100 ribu", "250 ribu", "50000", "300 ribu", "75 ribu", "400000"],
    },
}


QUOTAS = {
    None: {
        "reservation_create": 220,
        "reservation_cancel": 150,
        "check_reservation": 100,
        "ask_opening_hours": 160,
        "ask_menu_general": 170,
        "ask_menu_dietary": 130,
        "ask_menu_item_details": 130,
        "ask_location": 80,
        "ask_contact": 80,
        "ask_payment_methods": 80,
        "ask_price": 170,
        "ask_takeaway_delivery": 70,
        "ask_event": 50,
        "ask_facilities": 70,
        "ask_accessibility": 50,
        "ask_entertainment": 40,
        "greeting": 25,
        "thanks": 20,
        "goodbye": 20,
        "unknown": 25,
    },
    "WF_RESERVATION_CREATE": {
        "reservation_create": 280,
        "cancel": 50,
        "unknown": 30,
    },
    "WF_RESERVATION_CANCEL": {
        "reservation_cancel": 180,
        "cancel": 50,
        "unknown": 20,
    },
    "WF_CHOICE": {
        "affirmative": 25,
        "negative": 25,
    },
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


def pick(rng: random.Random, values: dict[str, list[str]], key: str) -> str:
    return rng.choice(values[key])


def price_condition(rng: random.Random, values: dict[str, list[str]]) -> tuple[str, list[tuple[str, str]]]:
    comparator = pick(rng, values, "price_comparator")
    amount = pick(rng, values, "price_amount")
    return f"{comparator} {amount}", [("price_comparator", comparator), ("price_amount", amount)]


def phrase_with_optional_wrapper(
    rng: random.Random,
    core: str,
    prefixes: list[str],
    suffixes: list[str],
) -> str:
    prefix = rng.choice(prefixes)
    suffix = rng.choice(suffixes)
    text = compose_phrase(prefix, core, suffix)
    return re.sub(r"\s+", " ", text)


def compose_phrase(prefix: str, core: str, suffix: str) -> str:
    if not uses_english_wrappers(prefixes=[prefix], suffixes=[suffix]):
        return f"{prefix}{core}{suffix}".strip()

    core = core.strip()
    prefix = prefix.strip()
    suffix = suffix.strip()

    if is_name_only(core):
        return core

    if prefix in {"I want to know", "could you tell me"}:
        return compose_english_indirect_question(prefix, core, suffix)

    if prefix == "please":
        return compose_english_please(core, suffix)

    if prefix == "can you":
        return compose_english_can_you(core, suffix)

    if prefix in {"I need to", "I want to", "I'd like to"}:
        return compose_english_request(prefix, core, suffix)

    if prefix in {"ok", "actually", "now"}:
        return compose_english_discourse_marker(prefix, core, suffix)

    return append_english_suffix(core, suffix)


def uses_english_wrappers(prefixes: list[str], suffixes: list[str]) -> bool:
    english_markers = {
        "please",
        "can you",
        "I want to",
        "I'd like to",
        "I need to",
        "could you tell me",
        "I want to know",
        "ok",
        "actually",
        "now",
    }
    normalized = {value.strip() for value in prefixes + suffixes}
    return bool(normalized & english_markers) or bool(normalized & {"today", "for tonight", "for dinner"})


def is_name_only(text: str) -> bool:
    return bool(re.fullmatch(r"[A-Z][A-Za-z'’-]+(?: [A-Z][A-Za-z'’-]+)+", text))


def append_english_suffix(text: str, suffix: str) -> str:
    if not suffix:
        return text
    if text.endswith("?"):
        return f"{text} please" if suffix == "please" else text
    if text.endswith("."):
        return text
    if suffix in {"today", "for tonight"} and ("today" in text or "tonight" in text):
        return text
    return f"{text} {suffix}"


def compose_english_please(core: str, suffix: str) -> str:
    if core.startswith(("Can ", "Could ", "Do ", "Does ", "Is ", "Are ", "What ", "Which ", "How ")):
        text = f"Please, {core[0].lower()}{core[1:]}"
    elif core.startswith(("I want ", "I need ", "I'd like ")):
        text = core
    else:
        text = f"Please {core}"
    return append_english_suffix(text, suffix)


def compose_english_can_you(core: str, suffix: str) -> str:
    if core.startswith(("I want ", "I need ", "I'd like ", "it is under", "reference ", "for ")):
        return append_english_suffix(core, suffix)
    if core.startswith(("book ", "cancel ")):
        return append_english_suffix(f"Can you {core}", suffix)
    if core.startswith(("Can ", "Could ", "Do ", "Does ", "Is ", "Are ", "What ", "Which ", "How ")):
        return append_english_suffix(core, suffix)
    return append_english_suffix(core, suffix)


def compose_english_request(prefix: str, core: str, suffix: str) -> str:
    if core.startswith(("I want ", "I need ", "I'd like ", "it is under", "reference ", "for ")):
        return append_english_suffix(core, suffix)
    if core.startswith(("Can ", "Could ", "Do ", "Does ", "Is ", "Are ", "What ", "Which ", "How ", "My email ")):
        return append_english_suffix(core, suffix)
    return append_english_suffix(f"{prefix} {core}", suffix)


def compose_english_discourse_marker(prefix: str, core: str, suffix: str) -> str:
    if prefix == "now" and core.startswith(("Can ", "Could ", "Do ", "Does ", "Is ", "Are ", "What ", "Which ", "How ")):
        return append_english_suffix(core, suffix)
    marker = "OK" if prefix == "ok" else prefix.capitalize()
    if core:
        return append_english_suffix(f"{marker}, {core[0].lower()}{core[1:]}", suffix)
    return marker


def compose_english_indirect_question(prefix: str, core: str, suffix: str) -> str:
    indirect = indirect_english_question(core)
    if indirect is None:
        return append_english_suffix(core, suffix)
    return append_english_suffix(f"{prefix} {indirect}", suffix)


def indirect_english_question(core: str) -> str | None:
    text = core.rstrip("?")
    lowered = text[0].lower() + text[1:] if text else text
    replacements = [
        (r"^Can you (.+)$", r"whether you can \1"),
        (r"^Could you (.+)$", r"whether you could \1"),
        (r"^Do you (.+)$", r"whether you \1"),
        (r"^Does the (.+) contain (.+)$", r"whether the \1 contains \2"),
        (r"^Is there (.+)$", r"whether there is \1"),
        (r"^Is the restaurant (.+)$", r"whether the restaurant is \1"),
        (r"^Is the (.+) (under|less than|below|greater than|more than|over) (.+)$", r"whether the \1 is \2 \3"),
        (r"^Are you (.+)$", r"whether you are \1"),
        (r"^What time are you open$", r"what time you are open"),
        (r"^What time do you close (.+)$", r"what time you close \1"),
        (r"^What is in the (.+)$", r"what is in the \1"),
        (r"^What costs (.+)$", r"what costs \1"),
        (r"^Which dishes are (.+)$", r"which dishes are \1"),
        (r"^Which dish contains (.+)$", r"which dish contains \1"),
        (r"^How much is the (.+)$", r"how much the \1 is"),
    ]
    for pattern, replacement in replacements:
        if re.match(pattern, text):
            return re.sub(pattern, replacement, text)
    if text.startswith("My email is "):
        return None
    if text.startswith(("I ", "Please ")):
        return None
    return lowered


def reservation_create_row(rng: random.Random, lang: str, task: str | None) -> dict[str, Any]:
    values = VALUES[lang]
    person = pick(rng, values, "person")
    date = pick(rng, values, "date")
    time = pick(rng, values, "time")
    people = pick(rng, values, "people_count")
    if lang == "en":
        templates = [
            (f"book a table for {people} {date} at {time} under {person}", [("people_count", people), ("date", date), ("time", time), ("person", person)]),
            (f"I need a reservation {date} at {time} for {people} under {person}", [("date", date), ("time", time), ("people_count", people), ("person", person)]),
            (f"{person} for {people} {date} at {time}", [("person", person), ("people_count", people), ("date", date), ("time", time)]),
            (f"for {people} {date} at {time}", [("people_count", people), ("date", date), ("time", time)]),
            (person, [("person", person)]),
            (f"{date} at {time}", [("date", date), ("time", time)]),
            (people, [("people_count", people)]),
        ]
        prefixes = ["", "please ", "can you ", "I want to ", "I'd like to "]
        suffixes = ["", ".", " please", " for dinner", " if available"]
    else:
        templates = [
            (f"pesan meja untuk {people} {date} jam {time} atas nama {person}", [("people_count", people), ("date", date), ("time", time), ("person", person)]),
            (f"saya mau reservasi {date} jam {time} untuk {people} atas nama {person}", [("date", date), ("time", time), ("people_count", people), ("person", person)]),
            (f"{person} untuk {people} {date} jam {time}", [("person", person), ("people_count", people), ("date", date), ("time", time)]),
            (f"untuk {people} {date} jam {time}", [("people_count", people), ("date", date), ("time", time)]),
            (person, [("person", person)]),
            (f"{date} jam {time}", [("date", date), ("time", time)]),
            (people, [("people_count", people)]),
        ]
        prefixes = ["", "tolong ", "bisa ", "saya ingin ", "mohon "]
        suffixes = ["", ".", " ya", " untuk makan malam", " kalau tersedia"]

    core, entities = rng.choice(templates)
    return row(phrase_with_optional_wrapper(rng, core, prefixes, suffixes), "reservation_create", lang, entities, task)


def reservation_cancel_row(rng: random.Random, lang: str, task: str | None) -> dict[str, Any]:
    values = VALUES[lang]
    person = pick(rng, values, "person")
    date = pick(rng, values, "date")
    reference = pick(rng, values, "reservation_reference")
    if lang == "en":
        templates = [
            (f"cancel reservation {reference}", [("reservation_reference", reference)]),
            (f"I want to cancel my booking with reference {reference}", [("reservation_reference", reference)]),
            (f"reference {reference}", [("reservation_reference", reference)]),
            (f"it is under {person} {date}", [("person", person), ("date", date)]),
            (f"cancel the reservation under {person} {date} with reference {reference}", [("person", person), ("date", date), ("reservation_reference", reference)]),
        ]
        prefixes = ["", "please ", "can you ", "I need to "]
        suffixes = ["", ".", " please", " now"]
    else:
        templates = [
            (f"batalkan reservasi {reference}", [("reservation_reference", reference)]),
            (f"saya mau membatalkan reservasi dengan referensi {reference}", [("reservation_reference", reference)]),
            (f"referensi {reference}", [("reservation_reference", reference)]),
            (f"atas nama {person} {date}", [("person", person), ("date", date)]),
            (f"batalkan reservasi atas nama {person} {date} dengan referensi {reference}", [("person", person), ("date", date), ("reservation_reference", reference)]),
        ]
        prefixes = ["", "tolong ", "bisa ", "saya perlu "]
        suffixes = ["", ".", " ya", " sekarang"]

    core, entities = rng.choice(templates)
    return row(phrase_with_optional_wrapper(rng, core, prefixes, suffixes), "reservation_cancel", lang, entities, task)


def static_row(rng: random.Random, lang: str, intent: str, task: str | None) -> dict[str, Any]:
    text_bank = {
        "en": {
            "greeting": ["Hello", "Hi", "Good evening", "Hey there", "Good morning", "Hi restaurant team"],
            "thanks": ["Thank you", "Thanks a lot", "I appreciate it", "Many thanks"],
            "goodbye": ["Goodbye", "Bye", "See you later", "Talk to you soon"],
            "affirmative": ["Yes", "Yes please", "That is correct", "I confirm", "Correct"],
            "negative": ["No", "No thanks", "That is not right", "I do not confirm", "Incorrect"],
            "cancel": ["cancel this flow", "stop the current request", "forget this request", "cancel the workflow"],
            "unknown": ["Can you book me a flight?", "Maybe later", "I need a taxi", "What is the weather?", "Play some music"],
        },
        "id": {
            "greeting": ["Halo", "Hai", "Selamat malam", "Selamat pagi", "Halo tim restoran", "Hai semua"],
            "thanks": ["Terima kasih", "Makasih banyak", "Saya menghargainya", "Terima kasih ya"],
            "goodbye": ["Sampai jumpa", "Dadah", "Sampai nanti", "Sampai bertemu lagi"],
            "affirmative": ["Ya", "Ya silakan", "Itu benar", "Saya konfirmasi", "Benar"],
            "negative": ["Tidak", "Tidak terima kasih", "Itu tidak benar", "Saya tidak konfirmasi", "Salah"],
            "cancel": ["batalkan alur ini", "hentikan permintaan ini", "lupakan permintaan ini", "batalkan workflow"],
            "unknown": ["Bisa pesankan tiket pesawat?", "Mungkin nanti", "Saya butuh taksi", "Bagaimana cuacanya?", "Putar musik"],
        },
    }
    prefixes = ["", "please ", "ok ", "actually ", "now "] if lang == "en" else ["", "tolong ", "baik ", "sebenarnya ", "sekarang "]
    suffixes = ["", ".", " please", " thanks", " for now"] if lang == "en" else ["", ".", " ya", " terima kasih", " dulu"]
    core = rng.choice(text_bank[lang][intent])
    return row(phrase_with_optional_wrapper(rng, core, prefixes, suffixes), intent, lang, task=task)


def informational_row(rng: random.Random, lang: str, intent: str) -> dict[str, Any]:
    values = VALUES[lang]
    condition, condition_entities = price_condition(rng, values)
    menu_item = pick(rng, values, "menu_item")
    price_item = pick(rng, values, "price_item")
    location = pick(rng, values, "location")
    phone = pick(rng, values, "phone")
    email = pick(rng, values, "email")
    dietary = pick(rng, values, "dietary_requirement")
    allergen = pick(rng, values, "allergen")
    facility = pick(rng, values, "facility")
    payment = pick(rng, values, "payment_method")
    reference = pick(rng, values, "reservation_reference")
    date = pick(rng, values, "date")
    time = pick(rng, values, "time")

    if lang == "en":
        templates = {
            "check_reservation": [
                (f"Do I have a reservation with reference {reference}?", [("reservation_reference", reference)]),
                (f"Can you check booking {reference}?", [("reservation_reference", reference)]),
            ],
            "ask_opening_hours": [
                ("What time are you open?", []),
                (f"Are you open {date} at {time}?", [("date", date), ("time", time)]),
                (f"What time do you close {date}?", [("date", date)]),
            ],
            "ask_menu_general": [
                ("Can I see the menu?", []),
                (f"Can I have the {price_item}?", [("price_item", price_item)]),
                (f"Do you have dishes {condition}?", condition_entities),
                (f"Show me meals {condition}", condition_entities),
            ],
            "ask_menu_dietary": [
                (f"Do you have {dietary} dishes?", [("dietary_requirement", dietary)]),
                (f"Which dishes are {dietary}?", [("dietary_requirement", dietary)]),
            ],
            "ask_menu_item_details": [
                (f"Does the {menu_item} contain {allergen}?", [("menu_item", menu_item), ("allergen", allergen)]),
                (f"What is in the {menu_item}?", [("menu_item", menu_item)]),
                (f"Which dish contains {allergen}?", [("allergen", allergen)]),
            ],
            "ask_location": [
                ("Can you share your address?", []),
                (f"Are you near {location}?", [("location", location)]),
            ],
            "ask_contact": [
                ("Could you provide your telephone number?", []),
                (f"Can you contact me at {phone}?", [("phone", phone)]),
                (f"My email is {email}", [("email", email)]),
            ],
            "ask_payment_methods": [
                (f"Can I pay by {payment}?", [("payment_method", payment)]),
                ("Can I split the bill?", []),
            ],
            "ask_price": [
                (f"How much is the {price_item}?", [("price_item", price_item)]),
                (f"Do you have meals {condition}?", condition_entities),
                (f"What costs {condition}?", condition_entities),
                (f"Is the {menu_item} {condition}?", [("menu_item", menu_item), *condition_entities]),
            ],
            "ask_takeaway_delivery": [
                ("I would like a delivery order.", []),
                ("Do you offer takeout?", []),
                ("Can I get this meal delivered?", []),
                ("Is takeaway available?", []),
                ("Can I order food to go?", []),
                ("Do you deliver to nearby addresses?", []),
                ("Can I pick up an order?", []),
                ("Is delivery available tonight?", []),
            ],
            "ask_event": [
                ("Do you host birthday parties?", []),
                ("Can I organize a business event there?", []),
                (f"Do you have a {location} for an event?", [("location", location)]),
            ],
            "ask_facilities": [
                (f"Do you have {facility}?", [("facility", facility)]),
                (f"Is there {facility} near your restaurant?", [("facility", facility)]),
            ],
            "ask_accessibility": [
                ("Is the restaurant wheelchair accessible?", []),
                ("Do you have disabled-friendly access?", []),
                ("Is there step-free access?", []),
                ("Can a stroller enter easily?", []),
            ],
            "ask_entertainment": [
                ("Do you have live music?", []),
                ("Is there karaoke tonight?", []),
                ("Do you host concerts?", []),
                ("Is there a DJ on weekends?", []),
                ("Do you have any entertainment tonight?", []),
                ("Are there performances this week?", []),
            ],
        }
        prefixes = ["", "please ", "could you tell me ", "I want to know "]
        suffixes = ["", ".", " please", " today", " for tonight"]
    else:
        templates = {
            "check_reservation": [
                (f"Apakah saya punya reservasi dengan referensi {reference}?", [("reservation_reference", reference)]),
                (f"Bisa cek booking {reference}?", [("reservation_reference", reference)]),
            ],
            "ask_opening_hours": [
                ("Jam berapa buka?", []),
                (f"Apakah buka {date} jam {time}?", [("date", date), ("time", time)]),
                (f"Jam berapa tutup {date}?", [("date", date)]),
            ],
            "ask_menu_general": [
                ("Boleh lihat menunya?", []),
                (f"Boleh minta {price_item}?", [("price_item", price_item)]),
                (f"Ada makanan {condition}?", condition_entities),
                (f"Tampilkan menu {condition}", condition_entities),
            ],
            "ask_menu_dietary": [
                (f"Ada hidangan {dietary}?", [("dietary_requirement", dietary)]),
                (f"Menu mana yang {dietary}?", [("dietary_requirement", dietary)]),
            ],
            "ask_menu_item_details": [
                (f"Apakah {menu_item} mengandung {allergen}?", [("menu_item", menu_item), ("allergen", allergen)]),
                (f"Apa isi {menu_item}?", [("menu_item", menu_item)]),
                (f"Hidangan mana yang mengandung {allergen}?", [("allergen", allergen)]),
            ],
            "ask_location": [
                ("Bisa kirim alamatnya?", []),
                (f"Apakah restoran dekat {location}?", [("location", location)]),
            ],
            "ask_contact": [
                ("Boleh kasih nomor teleponnya?", []),
                (f"Bisa hubungi saya di {phone}?", [("phone", phone)]),
                (f"Email saya {email}", [("email", email)]),
            ],
            "ask_payment_methods": [
                (f"Bisa bayar pakai {payment}?", [("payment_method", payment)]),
                ("Bisa pisah tagihan?", []),
            ],
            "ask_price": [
                (f"Berapa harga {price_item}?", [("price_item", price_item)]),
                (f"Ada makanan {condition}?", condition_entities),
                (f"Apa yang harganya {condition}?", condition_entities),
                (f"Apakah {menu_item} {condition}?", [("menu_item", menu_item), *condition_entities]),
            ],
            "ask_takeaway_delivery": [
                ("Saya mau pesan delivery.", []),
                ("Apakah ada takeout?", []),
                ("Bisa dikirim ke rumah?", []),
                ("Apakah takeaway tersedia?", []),
                ("Bisa pesan untuk dibawa pulang?", []),
            ],
            "ask_event": [
                ("Apakah bisa untuk pesta ulang tahun?", []),
                ("Bisa buat acara kantor?", []),
                (f"Ada {location} untuk acara?", [("location", location)]),
            ],
            "ask_facilities": [
                (f"Ada {facility}?", [("facility", facility)]),
                (f"Ada {facility} dekat restoran?", [("facility", facility)]),
            ],
            "ask_accessibility": [
                ("Apakah restoran ramah kursi roda?", []),
                ("Apakah aksesnya ramah disabilitas?", []),
                ("Apakah ada akses tanpa tangga?", []),
                ("Apakah stroller mudah masuk?", []),
            ],
            "ask_entertainment": [
                ("Ada live music?", []),
                ("Ada karaoke malam ini?", []),
            ],
        }
        prefixes = ["", "tolong ", "bisa jelaskan ", "saya ingin tahu "]
        suffixes = ["", ".", " ya", " hari ini", " untuk malam ini"]

    core, entities = rng.choice(templates[intent])
    return row(phrase_with_optional_wrapper(rng, core, prefixes, suffixes), intent, lang, entities)


def generator_for(intent: str) -> Callable[[random.Random, str, str | None], dict[str, Any]]:
    if intent == "reservation_create":
        return reservation_create_row
    if intent == "reservation_cancel":
        return reservation_cancel_row
    if intent in {"greeting", "thanks", "goodbye", "affirmative", "negative", "cancel", "unknown"}:
        return lambda rng, lang, task: static_row(rng, lang, intent, task)

    def generate(rng: random.Random, lang: str, task: str | None) -> dict[str, Any]:
        if task is not None:
            raise ValueError(f"Informational intent {intent} cannot be generated with task {task}")
        return informational_row(rng, lang, intent)

    return generate


def build_language_rows(lang: str) -> list[dict[str, Any]]:
    rng = random.Random(f"{SEED}:{lang}")
    rows: list[dict[str, Any]] = []
    seen: set[tuple[str, str | None, str, str]] = set()

    for task, intent_quotas in QUOTAS.items():
        for intent, quota in intent_quotas.items():
            generate = generator_for(intent)
            produced = 0
            for _ in range(quota * 80):
                candidate = generate(rng, lang, task)
                key = (candidate["lang"], candidate.get("task"), candidate["intent"], candidate["text"])
                if key in seen:
                    continue
                seen.add(key)
                rows.append(candidate)
                produced += 1
                if produced == quota:
                    break
            if produced != quota:
                raise RuntimeError(f"Could only generate {produced}/{quota} examples for {lang}:{task}:{intent}")

    if len(rows) != TARGET_ROWS_PER_LANGUAGE:
        raise RuntimeError(f"Expected {TARGET_ROWS_PER_LANGUAGE} rows for {lang}, got {len(rows)}")
    return rows


def build_english_rows() -> list[dict[str, Any]]:
    return build_language_rows("en")


def build_indonesian_rows() -> list[dict[str, Any]]:
    return build_language_rows("id")


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
