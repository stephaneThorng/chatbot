"""Generate deterministic English-only restaurant NLU datasets."""

from __future__ import annotations

import random
import re
from typing import Any

from nlu_training.config import load_config
from nlu_training.schema import load_jsonl, validate_examples, write_jsonl


SEED = 42
LANG = "en"
TARGET_TOTAL_ROWS = 1500

INTENT_ORDER = [
    "reservation_create",
    "reservation_cancel",
    "cancel",
    "check_reservation",
    "ask_opening_hours",
    "ask_menu_general",
    "ask_menu_dietary",
    "ask_menu_item_details",
    "ask_location",
    "ask_contact",
    "ask_payment_methods",
    "ask_price",
    "ask_takeaway_delivery",
    "ask_event",
    "ask_facilities",
    "ask_accessibility",
    "ask_entertainment",
    "greeting",
    "thanks",
    "goodbye",
    "affirmative",
    "negative",
    "unknown",
]

TASK_ORDER = {
    None: 0,
    "WF_RESERVATION_CREATE": 1,
    "WF_RESERVATION_CANCEL": 2,
    "WF_CHOICE": 3,
}

VALUES = {
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
        "Stephane",
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
    ],
    "phone": [
        "+33123456789",
        "01 23 45 67 89",
    ],
    "email": [
        "booking@example.com",
        "events@example.com",
        "hello@example.com",
    ],
    "dietary_requirement": [
        "vegan",
        "vegetarian",
        "halal",
        "gluten-free",
        "lactose-free",
        "dairy-free",
        "nut-free",
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
    ],
    "facility": [
        "parking",
        "wifi",
        "high chairs",
        "outdoor seating",
        "outdoor seats",
        "private room",
        "bike parking",
        "baby seat",
    ],
    "payment_method": [
        "credit card",
        "cash",
        "Apple Pay",
        "Google Pay",
        "Visa",
        "Mastercard",
        "contactless",
    ],
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
    "price_comparator": [
        "under",
        "less than",
        "below",
        "greater than",
        "more than",
        "over",
    ],
    "price_amount": [
        "20 euros",
        "$30",
        "15 euros",
        "25 dollars",
        "10 euros",
        "50 euros",
        "$45",
        "35 dollars",
    ],
}

QUOTAS = {
    None: {
        "reservation_create": 240,
        "reservation_cancel": 80,
        "check_reservation": 117,
        "ask_opening_hours": 75,
        "ask_menu_general": 113,
        "ask_menu_dietary": 85,
        "ask_menu_item_details": 85,
        "ask_location": 40,
        "ask_contact": 30,
        "ask_payment_methods": 35,
        "ask_price": 115,
        "ask_takeaway_delivery": 30,
        "ask_event": 25,
        "ask_facilities": 40,
        "ask_accessibility": 15,
        "ask_entertainment": 15,
        "greeting": 30,
        "thanks": 20,
        "goodbye": 35,
        "unknown": 30,
    },
    "WF_RESERVATION_CREATE": {
        "reservation_create": 120,
        "cancel": 20,
        "unknown": 10,
    },
    "WF_RESERVATION_CANCEL": {
        "reservation_cancel": 45,
        "cancel": 10,
        "unknown": 5,
    },
    "WF_CHOICE": {
        "affirmative": 18,
        "negative": 17,
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
    entities: list[tuple[str, str]] | None = None,
    task: str | None = None,
) -> dict[str, Any]:
    normalized_text = normalize_text(text)
    payload: dict[str, Any] = {
        "domain": "restaurant",
        "lang": LANG,
        "intent": intent,
    }
    if task is not None:
        payload["task"] = task
    payload["entities"] = span_entities(normalized_text, entities or [])
    payload["text"] = normalized_text
    return payload


def normalize_text(text: str) -> str:
    text = re.sub(r"\s+", " ", text.strip())
    text = text.replace(" ,", ",").replace(" .", ".").replace(" ?", "?")
    return text


def pick(rng: random.Random, key: str) -> str:
    return rng.choice(VALUES[key])


def people_count_digits() -> list[str]:
    digits: list[str] = []
    for value in VALUES["people_count"]:
        match = re.search(r"\d+", value)
        if not match:
            raise ValueError(f"Missing numeric value in people count: {value}")
        digits.append(match.group(0))
    return digits


def price_condition(rng: random.Random) -> tuple[str, list[tuple[str, str]]]:
    comparator = pick(rng, "price_comparator")
    amount = pick(rng, "price_amount")
    return f"{comparator} {amount}", [
        ("price_comparator", comparator),
        ("price_amount", amount),
    ]


def reservation_create_row(rng: random.Random, task: str | None) -> dict[str, Any]:
    person = pick(rng, "person")
    date = pick(rng, "date")
    time = pick(rng, "time")
    people = pick(rng, "people_count")
    people_digits = re.search(r"\d+", people).group(0)

    if task == "WF_RESERVATION_CREATE":
        templates = [
            ("{person}", [("person", person)]),
            ("{date}", [("date", date)]),
            ("{time}", [("time", time)]),
            ("{people_digits}", [("people_count", people_digits)]),
            ("for {people}", [("people_count", people)]),
            ("for {people_digits}", [("people_count", people_digits)]),
            ("{date} at {time}", [("date", date), ("time", time)]),
            (
                "{person} for {people}",
                [("person", person), ("people_count", people)],
            ),
            (
                "{person} {date} at {time}",
                [("person", person), ("date", date), ("time", time)],
            ),
            (
                "{date} at {time} for {people}",
                [("date", date), ("time", time), ("people_count", people)],
            ),
            (
                "{person} for {people} {date} at {time}",
                [
                    ("person", person),
                    ("people_count", people),
                    ("date", date),
                    ("time", time),
                ],
            ),
        ]
    else:
        templates = [
            ("book", []),
            ("book a reservation", []),
            ("book a table", []),
            ("book please", []),
            ("i want to book", []),
            ("please book a table for me", []),
            ("i want to make a reservation", []),
            ("i want to book a table", []),
            ("i need a reservation", []),
            ("i want to reserve", []),
            ("reserve a table", []),
            ("can i book", []),
            ("can you book a table for me", []),
            ("i would like to reserve a table", []),
            ("help me book a reservation", []),
            ("{person}", [("person", person)]),
            ("for {people}", [("people_count", people)]),
            ("{date}", [("date", date)]),
            ("at {time}", [("time", time)]),
            ("{date} at {time}", [("date", date), ("time", time)]),
            (
                "book a table for {people}",
                [("people_count", people)],
            ),
            (
                "book a table for {people} {date}",
                [("people_count", people), ("date", date)],
            ),
            (
                "book a table for {people} {date} at {time}",
                [("people_count", people), ("date", date), ("time", time)],
            ),
            (
                "book a table for {people} {date} at {time} under {person}",
                [
                    ("people_count", people),
                    ("date", date),
                    ("time", time),
                    ("person", person),
                ],
            ),
            (
                "please reserve a table under {person} for {people} {date} at {time}",
                [
                    ("person", person),
                    ("people_count", people),
                    ("date", date),
                    ("time", time),
                ],
            ),
            (
                "i need a reservation {date} at {time} for {people} under {person}",
                [
                    ("date", date),
                    ("time", time),
                    ("people_count", people),
                    ("person", person),
                ],
            ),
            (
                "reserve a table for {people} under {person}",
                [("people_count", people), ("person", person)],
            ),
            (
                "i want a table for {people} {date} at {time}",
                [("people_count", people), ("date", date), ("time", time)],
            ),
        ]

    template, entities = rng.choice(templates)
    return row(
        template.format(
            person=person,
            date=date,
            time=time,
            people=people,
            people_digits=people_digits,
        ),
        "reservation_create",
        entities,
        task,
    )


def reservation_cancel_row(rng: random.Random, task: str | None) -> dict[str, Any]:
    person = pick(rng, "person")
    date = pick(rng, "date")
    reference = pick(rng, "reservation_reference")

    if task == "WF_RESERVATION_CANCEL":
        templates = [
            ("{reference}", [("reservation_reference", reference)]),
            ("under {person}", [("person", person)]),
            ("for {date}", [("date", date)]),
            (
                "{reference} under {person}",
                [("reservation_reference", reference), ("person", person)],
            ),
            (
                "{reference} for {date}",
                [("reservation_reference", reference), ("date", date)],
            ),
        ]
    else:
        templates = [
            ("cancel my reservation", []),
            ("cancel my booking", []),
            ("i want to cancel a reservation", []),
            ("please cancel my booking", []),
            ("cancel reservation {reference}", [("reservation_reference", reference)]),
            (
                "cancel booking {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "i want to cancel my reservation with reference {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "cancel the reservation under {person}",
                [("person", person)],
            ),
            (
                "cancel the reservation under {person} for {date}",
                [("person", person), ("date", date)],
            ),
            (
                "cancel reservation {reference} under {person}",
                [("reservation_reference", reference), ("person", person)],
            ),
        ]

    template, entities = rng.choice(templates)
    return row(
        template.format(reference=reference, person=person, date=date),
        "reservation_cancel",
        entities,
        task,
    )


def static_row(rng: random.Random, intent: str, task: str | None) -> dict[str, Any]:
    bank = {
        "greeting": [
            "hello",
            "hello!",
            "hi",
            "hi!",
            "good morning",
            "good evening",
            "hey",
            "hello there",
            "hi there",
            "hey there",
            "hello bot",
            "hello restaurant team",
            "hi restaurant team",
            "good afternoon",
            "hello again",
            "hi again",
            "hey restaurant",
            "good day",
            "hello team",
            "hi team",
            "morning",
            "evening",
            "hello, can you help me",
            "hi, can you help me",
            "hello restaurant",
            "hi restaurant",
            "greetings",
            "greetings restaurant",
            "hey, can you help me",
            "hello my friend",
            "hi my friend",
        ],
        "thanks": [
            "thank you",
            "thanks",
            "thanks a lot",
            "i appreciate it",
            "many thanks",
            "thank you very much",
            "thanks for the help",
            "thanks, that helps",
            "thank you for your help",
            "great, thanks",
            "perfect, thanks",
            "thanks very much",
            "that is helpful, thanks",
            "appreciate it",
            "thank you, that is clear",
            "thanks so much",
            "thank you so much",
            "thanks for that",
            "that helps a lot, thanks",
            "many thanks for your help",
        ],
        "goodbye": [
            "goodbye",
            "good bye",
            "bye",
            "bye!",
            "see you later",
            "talk to you soon",
            "bye for now",
            "see you soon",
            "have a good day",
            "have a nice evening",
            "talk later",
            "catch you later",
            "bye, thanks",
            "good night",
            "i will come back later",
            "speak soon",
            "bye and thanks",
            "farewell",
            "see you next time",
            "until next time",
            "talk again soon",
            "bye restaurant team",
            "goodbye restaurant team",
            "bye team",
            "goodbye team",
            "see you around",
            "i will talk to you later",
            "thank you, goodbye",
            "okay goodbye",
            "okay, bye",
            "bye now",
            "leaving now, goodbye",
            "see you later, thanks",
            "good night and goodbye",
            "good bye for now",
            "talk soon, bye",
        ],
        "affirmative": [
            "y",
            "yes",
            "Yes",
            "yes please",
            "that is correct",
            "i confirm",
            "correct",
            "yeah",
            "yep",
            "ok",
            "okay",
            "yes, that is right",
            "please confirm it",
            "go ahead",
            "that works",
            "confirmed",
            "yes, confirm it",
            "sure, confirm",
        ],
        "negative": [
            "n",
            "no",
            "No",
            "nope",
            "nah",
            "no thanks",
            "that is not right",
            "i do not confirm",
            "incorrect",
            "no, that is wrong",
            "please do not confirm",
            "stop, that is not correct",
            "no, cancel it",
            "that does not work",
            "not correct",
            "no, that is not my request",
            "do not confirm that",
        ],
        "cancel": [
            "cancel",
            "cancel this flow",
            "stop this request",
            "forget this request",
            "cancel the current request",
            "stop the current workflow",
            "drop this booking request",
            "please cancel this",
            "abort this request",
            "never mind, cancel it",
            "cancel the reservation flow",
            "stop the booking flow",
            "i want to cancel this process",
            "cancel the current flow",
            "end this request",
            "stop this booking request",
            "please stop this flow",
            "cancel the ongoing request",
            "abort the current flow",
            "i want to stop this request",
        ],
        "unknown": [
            "i am so happy",
            "tell me a joke",
            "what is the weather",
            "i need a taxi",
            "play some music",
            "that sounds nice",
            "i am just chatting",
            "can you book me a flight",
            "show me the news",
            "i want a hotel room",
            "this is random",
            "do you like movies",
            "i am excited today",
            "i like this place",
            "sing a song",
            "how is traffic",
            "i need a haircut",
            "tell me something fun",
            "this is great",
            "i feel happy today",
            "can you drive me home",
            "what time is it",
            "i am bored",
            "let's talk about sports",
            "what is your favorite color",
            "tell me a story",
            "what do you think about movies",
            "can you dance",
            "i feel fantastic today",
            "tell me something interesting",
        ],
    }
    return row(rng.choice(bank[intent]), intent, task=task)


def informational_row(rng: random.Random, intent: str) -> dict[str, Any]:
    menu_item = pick(rng, "menu_item")
    price_item = pick(rng, "price_item")
    location = pick(rng, "location")
    phone = pick(rng, "phone")
    email = pick(rng, "email")
    dietary = pick(rng, "dietary_requirement")
    allergen = pick(rng, "allergen")
    facility = pick(rng, "facility")
    payment = pick(rng, "payment_method")
    reference = pick(rng, "reservation_reference")
    date = pick(rng, "date")
    time = pick(rng, "time")
    condition, condition_entities = price_condition(rng)

    templates = {
        "check_reservation": [
            ("can i check my reservation", []),
            ("i want to check my reservation", []),
            ("check my booking", []),
            ("check my reservation status", []),
            ("can you look up my reservation", []),
            ("look up my booking", []),
            ("find my reservation", []),
            ("i need to check a reservation", []),
            ("show my booking status", []),
            ("i want to look up a booking", []),
            (
                "check booking {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "can you check booking {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "do i have a reservation with reference {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "check reservation {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "look up reservation {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "find booking {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "search booking {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "is reservation {reference} still active",
                [("reservation_reference", reference)],
            ),
            (
                "what is the status of booking {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "can i check booking {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "tell me about reservation {reference}",
                [("reservation_reference", reference)],
            ),
            (
                "i want to see reservation {reference}",
                [("reservation_reference", reference)],
            ),
        ],
        "ask_opening_hours": [
            ("what time are you open", []),
            ("what are your opening hours", []),
            ("what time do you close", []),
            ("are you open {date} at {time}", [("date", date), ("time", time)]),
        ],
        "ask_menu_general": [
            ("can you give me the menu", []),
            ("show me the menu", []),
            ("i want the menu", []),
            ("what is on the menu", []),
            ("can i see the menu", []),
            ("show me your menu", []),
            ("what food do you have", []),
            ("what dishes do you serve", []),
            ("what can i order", []),
            ("show me the {price_item}", [("price_item", price_item)]),
            ("what is in the {price_item}", [("price_item", price_item)]),
            ("can i see the {price_item}", [("price_item", price_item)]),
            ("tell me about the {menu_item}", [("menu_item", menu_item)]),
            ("do you have meals {condition}", condition_entities),
            ("show me dishes {condition}", condition_entities),
            ("what menu items are {condition}", condition_entities),
            ("show me options {condition}", condition_entities),
        ],
        "ask_menu_dietary": [
            ("do you have {dietary} dishes", [("dietary_requirement", dietary)]),
            ("which dishes are {dietary}", [("dietary_requirement", dietary)]),
            ("show me {dietary} options", [("dietary_requirement", dietary)]),
            ("is there anything {dietary}", [("dietary_requirement", dietary)]),
            ("what {dietary} dishes do you have", [("dietary_requirement", dietary)]),
            ("show me {dietary} food", [("dietary_requirement", dietary)]),
            ("can i get a {dietary} meal", [("dietary_requirement", dietary)]),
            ("which menu items are {dietary}", [("dietary_requirement", dietary)]),
            ("what {dietary} options are available", [("dietary_requirement", dietary)]),
            ("do you serve {dietary} food", [("dietary_requirement", dietary)]),
            ("i need a {dietary} option", [("dietary_requirement", dietary)]),
            ("are there any {dietary} meals", [("dietary_requirement", dietary)]),
            ("can you suggest {dietary} dishes", [("dietary_requirement", dietary)]),
            ("what can i eat if i need {dietary} food", [("dietary_requirement", dietary)]),
        ],
        "ask_menu_item_details": [
            (
                "does the {menu_item} contain {allergen}",
                [("menu_item", menu_item), ("allergen", allergen)],
            ),
            ("what is in the {menu_item}", [("menu_item", menu_item)]),
            ("which dish contains {allergen}", [("allergen", allergen)]),
            ("tell me about the {menu_item}", [("menu_item", menu_item)]),
        ],
        "ask_location": [
            ("what is your location", []),
            ("what is your address", []),
            ("where are you located", []),
            ("are you near {location}", [("location", location)]),
            ("where is the restaurant", []),
            ("can you share the address", []),
            ("how do i find the restaurant", []),
            ("what is the restaurant address", []),
            ("tell me where you are", []),
            ("can you tell me the location", []),
            ("is the restaurant close to {location}", [("location", location)]),
            ("are you located near {location}", [("location", location)]),
            ("is your restaurant by {location}", [("location", location)]),
            ("are you around {location}", [("location", location)]),
        ],
        "ask_contact": [
            ("how can i contact you", []),
            ("what is your phone number", []),
            ("what is your email address", []),
            ("how do i reach you", []),
            ("what is the best way to contact you", []),
            ("can i call the restaurant", []),
            ("can i email the restaurant", []),
            ("how can i get in touch", []),
            ("give me your contact details", []),
            ("i need your contact information", []),
            ("what contact details should i use", []),
            ("how do customers contact you", []),
            ("what number should i call", []),
            ("what email should i use", []),
            ("can you share your contact information", []),
            ("tell me your contact details", []),
            ("what is the restaurant phone number", []),
            ("what is the restaurant email", []),
            ("how can i reach the booking team", []),
            ("can i get your booking contact", []),
            ("can i reach you at {phone}", [("phone", phone)]),
            ("can i contact you at {email}", [("email", email)]),
            ("is {phone} the right number", [("phone", phone)]),
            ("is {email} the right email", [("email", email)]),
        ],
        "ask_payment_methods": [
            ("what payment methods do you accept", []),
            ("can i pay by {payment}", [("payment_method", payment)]),
            ("do you take {payment}", [("payment_method", payment)]),
            ("can i split the bill", []),
            ("do you accept {payment}", [("payment_method", payment)]),
            ("is {payment} accepted", [("payment_method", payment)]),
            ("can i use {payment}", [("payment_method", payment)]),
        ],
        "ask_price": [
            ("how much is the {price_item}", [("price_item", price_item)]),
            ("what is the price of the {menu_item}", [("menu_item", menu_item)]),
            ("what costs {condition}", condition_entities),
            ("do you have dishes {condition}", condition_entities),
            (
                "is the {menu_item} {condition}",
                [("menu_item", menu_item), *condition_entities],
            ),
        ],
        "ask_takeaway_delivery": [
            ("do you offer takeaway", []),
            ("do you offer delivery", []),
            ("can i order food to go", []),
            ("is takeaway available", []),
            ("can i pick up an order", []),
            ("is delivery available tonight", []),
            ("do you do takeout", []),
            ("can i get delivery", []),
            ("can i place a takeaway order", []),
            ("can i collect an order", []),
            ("is takeout possible", []),
            ("do you deliver food", []),
            ("is pickup available", []),
            ("can i order for pickup", []),
            ("do you have takeaway service", []),
            ("can i take food away", []),
            ("is to-go ordering available", []),
            ("can i place a delivery order", []),
            ("do you have pickup service", []),
            ("can i get a takeaway order", []),
            ("do you offer food delivery", []),
            ("can i make a takeout order", []),
            ("can i order takeaway tonight", []),
            ("can i arrange a pickup order", []),
            ("is a takeaway order possible", []),
            ("do you support pickup", []),
            ("do you support delivery", []),
            ("can i order this for takeaway", []),
            ("can i have this delivered", []),
            ("is collection available", []),
        ],
        "ask_event": [
            ("do you host birthday parties", []),
            ("can i organize a business event there", []),
            ("do you have a {location} for an event", [("location", location)]),
            ("can i book the restaurant for a private event", []),
            ("do you host private dinners", []),
            ("can i reserve a {location} for an event", [("location", location)]),
            ("can i hold an event at the restaurant", []),
            ("do you organize corporate dinners", []),
            ("can i use the venue for a celebration", []),
            ("do you host anniversary dinners", []),
            ("is the {location} available for events", [("location", location)]),
            ("can i book the {location} for a private dinner", [("location", location)]),
            ("do you offer event space in the {location}", [("location", location)]),
            ("can i host a party there", []),
            ("do you take private event bookings", []),
            ("is the restaurant available for celebrations", []),
            ("can i reserve space for an event", []),
            ("do you host company events", []),
            ("can i arrange a private dinner there", []),
            ("is the {location} suitable for a private event", [("location", location)]),
            ("can the {location} be reserved for a celebration", [("location", location)]),
        ],
        "ask_facilities": [
            ("do you have {facility}", [("facility", facility)]),
            ("is there {facility}", [("facility", facility)]),
            ("do you have parking", [("facility", "parking")]),
            ("do you have outdoor seats", [("facility", "outdoor seats")]),
            ("is {facility} available", [("facility", facility)]),
            ("do you offer {facility}", [("facility", facility)]),
            ("can i use {facility}", [("facility", facility)]),
            ("is {facility} provided", [("facility", facility)]),
        ],
        "ask_accessibility": [
            ("is the restaurant wheelchair accessible", []),
            ("do you have disabled-friendly access", []),
            ("is there step-free access", []),
            ("can a stroller enter easily", []),
            ("is the entrance accessible", []),
            ("can wheelchair users enter easily", []),
            ("do you have accessible access", []),
            ("is the restaurant accessible for disabled guests", []),
            ("is there an accessible entrance", []),
            ("can strollers get in easily", []),
            ("do you have step free access", []),
            ("is the venue stroller friendly", []),
            ("is the venue wheelchair friendly", []),
            ("do you support accessible entry", []),
            ("is the place accessible", []),
        ],
        "ask_entertainment": [
            ("do you have live music", []),
            ("is there a dj on weekends", []),
            ("do you have music", []),
            ("do you have any entertainment tonight", []),
            ("is there live music tonight", []),
            ("do you host performances", []),
            ("is there a dj tonight", []),
            ("do you have weekend entertainment", []),
            ("are there performances this week", []),
            ("do you play music in the evening", []),
            ("do you host live performances", []),
            ("is there any music tonight", []),
            ("do you offer evening entertainment", []),
            ("do you have a music program", []),
            ("do you have shows on weekends", []),
        ],
    }

    template, entities = rng.choice(templates[intent])
    return row(
        template.format(
            menu_item=menu_item,
            price_item=price_item,
            location=location,
            phone=phone,
            email=email,
            dietary=dietary,
            allergen=allergen,
            facility=facility,
            payment=payment,
            reference=reference,
            date=date,
            time=time,
            condition=condition,
        ),
        intent,
        entities,
    )


def generator_for(intent: str):
    if intent == "reservation_create":
        return reservation_create_row
    if intent == "reservation_cancel":
        return reservation_cancel_row
    if intent in {"greeting", "thanks", "goodbye", "affirmative", "negative", "cancel", "unknown"}:
        return lambda rng, task: static_row(rng, intent, task)
    return lambda rng, task: informational_row(rng, intent)


def mandatory_rows_for(task: str | None, intent: str) -> list[dict[str, Any]]:
    if task is None and intent == "greeting":
        return [
            row("hello", "greeting"),
            row("hi", "greeting"),
            row("good morning", "greeting"),
            row("good evening", "greeting"),
        ]
    if task is None and intent == "goodbye":
        return [
            row("goodbye", "goodbye"),
            row("good bye", "goodbye"),
            row("bye", "goodbye"),
            row("see you later", "goodbye"),
            row("talk to you soon", "goodbye"),
        ]
    if task is None and intent == "thanks":
        return [
            row("thanks", "thanks"),
            row("thank you", "thanks"),
            row("thanks a lot", "thanks"),
        ]
    if task == "WF_RESERVATION_CREATE" and intent == "reservation_create":
        return [
            row("1", "reservation_create", [("people_count", "1")], task),
            row("2", "reservation_create", [("people_count", "2")], task),
            row("4", "reservation_create", [("people_count", "4")], task),
            row("6", "reservation_create", [("people_count", "6")], task),
            row("8", "reservation_create", [("people_count", "8")], task),
            row("10", "reservation_create", [("people_count", "10")], task),
            row("12", "reservation_create", [("people_count", "12")], task),
            row("for 4", "reservation_create", [("people_count", "4")], task),
            row("for 10", "reservation_create", [("people_count", "10")], task),
            row("for 4 people", "reservation_create", [("people_count", "4 people")], task),
            row("for 10 people", "reservation_create", [("people_count", "10 people")], task),
            row(
                "tomorrow at 7pm",
                "reservation_create",
                [("date", "tomorrow"), ("time", "7pm")],
                task,
            ),
        ]
    if task == "WF_CHOICE" and intent == "affirmative":
        return [
            row("y", "affirmative", task=task),
            row("yes", "affirmative", task=task),
            row("Yes", "affirmative", task=task),
            row("yes please", "affirmative", task=task),
            row("i confirm", "affirmative", task=task),
            row("confirmed", "affirmative", task=task),
            row("go ahead", "affirmative", task=task),
            row("okay", "affirmative", task=task),
        ]
    if task == "WF_CHOICE" and intent == "negative":
        return [
            row("n", "negative", task=task),
            row("no", "negative", task=task),
            row("No", "negative", task=task),
            row("nope", "negative", task=task),
            row("nah", "negative", task=task),
            row("no thanks", "negative", task=task),
            row("that is not right", "negative", task=task),
            row("i do not confirm", "negative", task=task),
        ]
    return []


def build_english_rows() -> list[dict[str, Any]]:
    rng = random.Random(SEED)
    rows: list[dict[str, Any]] = []
    seen: set[tuple[str, str | None, str, str]] = set()

    for task, intent_quotas in QUOTAS.items():
        for intent in INTENT_ORDER:
            quota = intent_quotas.get(intent)
            if quota is None:
                continue
            generate = generator_for(intent)
            produced = 0

            for candidate in mandatory_rows_for(task, intent):
                key = (
                    candidate["lang"],
                    candidate.get("task"),
                    candidate["intent"],
                    candidate["text"],
                )
                if key in seen:
                    continue
                seen.add(key)
                rows.append(candidate)
                produced += 1
                if produced == quota:
                    break

            for _ in range(quota * 120):
                if produced == quota:
                    break
                candidate = generate(rng, task)
                key = (
                    candidate["lang"],
                    candidate.get("task"),
                    candidate["intent"],
                    candidate["text"],
                )
                if key in seen:
                    continue
                seen.add(key)
                rows.append(candidate)
                produced += 1
                if produced == quota:
                    break
            if produced != quota:
                raise RuntimeError(
                    f"Could only generate {produced}/{quota} examples for {task}:{intent}"
                )

    if len(rows) != TARGET_TOTAL_ROWS:
        raise RuntimeError(
            f"Expected {TARGET_TOTAL_ROWS} rows, got {len(rows)}"
        )
    return sort_rows(rows)


def sort_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    order = {intent: index for index, intent in enumerate(INTENT_ORDER)}
    return sorted(
        rows,
        key=lambda item: (
            order[item["intent"]],
            TASK_ORDER[item.get("task")],
            len(item["entities"]),
            item["text"].lower(),
        ),
    )


def split_rows(
    rows: list[dict[str, Any]],
) -> tuple[list[dict[str, Any]], list[dict[str, Any]], list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = {}
    for item in rows:
        grouped.setdefault(
            f"{item['intent']}:{item.get('task', '-')}",
            [],
        ).append(item)

    train: list[dict[str, Any]] = []
    validation: list[dict[str, Any]] = []
    eval_rows: list[dict[str, Any]] = []
    for key in sorted(grouped):
        group_rows = sort_rows(grouped[key])
        if len(group_rows) >= 3:
            validation.append(group_rows[0])
            eval_rows.append(group_rows[1])
            train.extend(group_rows[2:])
        elif len(group_rows) == 2:
            validation.append(group_rows[0])
            train.append(group_rows[1])
        else:
            train.extend(group_rows)

    return sort_rows(train), sort_rows(validation), sort_rows(eval_rows)


def main() -> None:
    config = load_config()
    rows = build_english_rows()
    train, validation, eval_rows = split_rows(rows)

    data_config = config["data"]
    write_jsonl(data_config["train"], train)
    write_jsonl(data_config["validation"], validation)
    write_jsonl(data_config["eval"], eval_rows)

    for path in (
        data_config["train"],
        data_config["validation"],
        data_config["eval"],
    ):
        examples = load_jsonl(path)
        validate_examples(examples, config)

    print(
        f"Generated {len(train)} train, {len(validation)} validation, {len(eval_rows)} eval examples."
    )


if __name__ == "__main__":
    main()
