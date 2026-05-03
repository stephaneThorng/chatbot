"""Generate a deterministic restaurant training dataset."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path
from typing import List, Sequence


OUTPUT_DIR = Path("training/data/restaurant")
INTENT_TARGETS = {
    "reservation_create": 760,
    "reservation_modify": 540,
    "reservation_cancel": 380,
    "reservation_status": 260,
    "menu_request": 640,
    "opening_hours": 560,
    "location_request": 540,
    "pricing_request": 540,
    "contact_request": 780,
    "unknown": 500,
}
EXPECTED_CORPUS_SIZE = 5500
EXPECTED_SPLIT_SIZES = {"train": 4400, "validation": 550, "eval": 550}
UNIQUENESS_SUFFIXES = [
    ", please",
    " for dinner",
    " for lunch",
    " this week",
    " for this weekend",
    " if possible",
    " when you can",
    " for planning",
    " for tonight",
    " for next week",
    " before I visit",
]


@dataclass(frozen=True, slots=True)
class EntityAnnotation:
    start: int
    end: int
    type: str


@dataclass(frozen=True, slots=True)
class Example:
    text: str
    intent: str
    entities: tuple[EntityAnnotation, ...]

    def to_payload(self) -> dict[str, object]:
        return {
            "text": self.text,
            "intent": self.intent,
            "entities": [
                {"start": entity.start, "end": entity.end, "type": entity.type}
                for entity in self.entities
            ],
        }


def build_example(intent: str, parts: Sequence[object]) -> Example:
    text_parts: list[str] = []
    entities: list[EntityAnnotation] = []
    cursor = 0
    for part in parts:
        if isinstance(part, tuple):
            entity_type, value = part
            start = cursor
            end = start + len(value)
            entities.append(EntityAnnotation(start=start, end=end, type=entity_type))
            text_parts.append(value)
            cursor = end
        else:
            value = str(part)
            text_parts.append(value)
            cursor += len(value)
    return Example(text="".join(text_parts), intent=intent, entities=tuple(entities))


def pick(values: Sequence[str], primary: int, secondary: int = 0, step: int = 1) -> str:
    """Pick a stable value from a sequence with two moving indices."""

    return values[(primary * step + secondary) % len(values)]


NAMES = [
    "Alex Carter",
    "Maya Chen",
    "Jordan Lee",
    "Priya Singh",
    "Noah Davis",
    "Lena Brooks",
    "Sam Rivera",
    "Olivia Reed",
    "Ethan Cole",
    "Chloe Martin",
]

PHONES = [
    "555-0101",
    "555-0102",
    "555-0103",
    "555-0104",
    "555-0105",
    "555-0106",
    "555-0107",
    "555-0108",
    "555-0109",
    "555-0110",
]

EMAILS = [
    "alex.carter@example.com",
    "maya.chen@example.com",
    "jordan.lee@example.com",
    "priya.singh@example.com",
    "noah.davis@example.com",
    "lena.brooks@example.com",
    "sam.rivera@example.com",
    "olivia.reed@example.com",
    "ethan.cole@example.com",
    "chloe.martin@example.com",
]

PEOPLE_COUNTS = ["2", "3", "4", "5", "6", "7", "8", "10"]
DATES = [
    "today",
    "tomorrow",
    "Friday",
    "Saturday",
    "Sunday",
    "next Monday",
    "next Tuesday",
    "June 12",
    "July 4",
    "August 18",
]
TIMES = [
    "5pm",
    "5:30pm",
    "6pm",
    "6:30pm",
    "7pm",
    "7:30pm",
    "8pm",
    "8:30pm",
    "9pm",
    "tomorrow evening",
]
MENU_TOPICS = [
    "vegan options",
    "kids menu",
    "dessert menu",
    "drink list",
    "gluten free dishes",
    "chef specials",
    "seafood options",
    "lunch menu",
    "wine pairings",
    "brunch menu",
]
PRICE_TOPICS = [
    "tasting menu",
    "brunch buffet",
    "kids meals",
    "wine pairing",
    "steak special",
    "private dining menu",
    "seafood platter",
    "happy hour snacks",
    "date night set menu",
    "family meal",
]
LOCATIONS = [
    "Downtown",
    "Main Street",
    "Riverside",
    "Old Town",
    "the train station",
    "City Center",
    "Pine Avenue",
    "the art museum",
    "the waterfront",
    "Market Square",
]
DAY_PHRASES = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
    "this weekend",
    "tomorrow",
    "next week",
]
MENU_CONTEXTS = [
    "before I visit",
    "for a birthday dinner",
    "for a work lunch",
    "for tonight",
    "before booking",
    "for my family",
    "for a date night",
    "for takeout planning",
    "for weekend dinner",
    "for a client meal",
]
HOUR_CONTEXTS = [
    "for dinner plans",
    "before I drive over",
    "for a late booking",
    "for lunch tomorrow",
    "for weekend planning",
    "for a family visit",
    "for a business dinner",
    "for a quick stop",
    "for holiday plans",
    "for tonight",
]
LOCATION_CONTEXTS = [
    "I'm driving in",
    "I'm taking the train",
    "I'm walking over",
    "I'm meeting friends there",
    "I'm visiting the area",
    "I'm planning parking",
    "I'm coming from downtown",
    "I'm staying nearby",
    "I'm using a rideshare",
    "I'm heading there after work",
]
PRICE_CONTEXTS = [
    "for budget planning",
    "before I reserve",
    "for a family outing",
    "for date night",
    "for a team dinner",
    "for birthday plans",
    "for lunch this week",
    "before ordering",
    "for weekend plans",
    "for a special occasion",
]
CONTACT_CONTEXTS = [
    "for reservation help",
    "for event questions",
    "for catering info",
    "for a same-day booking",
    "for private dining",
    "for feedback",
    "for allergy questions",
    "for a callback",
    "for a press inquiry",
    "for a delivery issue",
]
CONTACT_SUFFIXES = [
    "today",
    "this week",
    "for reservations",
    "for general questions",
    "before I visit",
    "for event details",
    "for quick help",
    "for planning",
    "for follow-up",
]
RESERVATION_STATUS_CONTEXTS = [
    "before I invite my guests",
    "before I leave home",
    "for planning",
    "for tonight",
    "before I call",
    "for my calendar",
    "before dinner",
    "for the group",
    "for my records",
    "before I arrive",
]
UNKNOWN_SINGLE_TOKENS = [
    "carrot",
    "banana",
    "pillow",
    "notebook",
    "lantern",
    "cactus",
    "penguin",
    "bicycle",
    "sunset",
    "marble",
]
UNKNOWN_SMALL_TALK = [
    "How am I doing today?",
    "I am feeling great right now.",
    "I am very very good at eating.",
    "Tell me something random.",
    "That is not what I meant.",
    "I just wanted to say something odd.",
    "This is unrelated to dinner planning.",
    "I am testing random conversation here.",
    "That feels a bit off topic.",
    "I am chatting without a restaurant question.",
]
UNKNOWN_VAGUE_FOLLOW_UPS = [
    "what else?",
    "anything more?",
    "other options?",
    "what about the rest?",
    "and then?",
    "go on?",
]
def reservation_create_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda c, d, t, n: build_example(
            "reservation_create",
            ["I do not have a reservation yet, so book a table for ", ("PEOPLE_COUNT", c), " on ", ("DATE", d), " at ", ("TIME", t), " under ", ("PERSON", n)],
        ),
        lambda c, d, t, n: build_example(
            "reservation_create",
            ["I'd like to make a first-time reservation for ", ("PEOPLE_COUNT", c), " people on ", ("DATE", d), " at ", ("TIME", t), " for ", ("PERSON", n)],
        ),
        lambda c, d, t, n: build_example(
            "reservation_create",
            ["Can you start a new table booking for me for ", ("PEOPLE_COUNT", c), " at ", ("TIME", t), " on ", ("DATE", d), " for ", ("PERSON", n)],
        ),
        lambda c, d, t, n: build_example(
            "reservation_create",
            ["This is a first booking request, not a change, for ", ("PEOPLE_COUNT", c), " on ", ("DATE", d), " at ", ("TIME", t), ", name ", ("PERSON", n)],
        ),
        lambda c, d, t, n: build_example(
            "reservation_create",
            ["Please create my first dinner reservation for ", ("PEOPLE_COUNT", c), " on ", ("DATE", d), " around ", ("TIME", t), " for ", ("PERSON", n)],
        ),
    ]
    noisy_templates = [
        lambda c, d, t, p: build_example(
            "reservation_create",
            ["I do not have any booking yet, and I need a brand new reservation for ", ("PEOPLE_COUNT", c), " on ", ("DATE", d), " at ", ("TIME", t), "; call me at ", ("PHONE", p)],
        ),
        lambda c, d, t, e: build_example(
            "reservation_create",
            ["Please book a fresh new reservation from scratch for ", ("PEOPLE_COUNT", c), " people on ", ("DATE", d), " at ", ("TIME", t), "; email ", ("EMAIL", e)],
        ),
        lambda c, d, t, p: build_example(
            "reservation_create",
            ["Please create a completely new booking for ", ("PEOPLE_COUNT", c), " on ", ("DATE", d), " at ", ("TIME", t), "; phone ", ("PHONE", p)],
        ),
        lambda c, d, t, e: build_example(
            "reservation_create",
            ["I'd like to open a brand new reservation because I have no booking yet for ", ("PEOPLE_COUNT", c), " on ", ("DATE", d), " around ", ("TIME", t), "; contact ", ("EMAIL", e)],
        ),
        lambda c, d, t, p: build_example(
            "reservation_create",
            ["Could I make a first table booking for ", ("PEOPLE_COUNT", c), " on ", ("DATE", d), " at ", ("TIME", t), "? reach me at ", ("PHONE", p)],
        ),
    ]
    standard_count = max(1, target_count - max(10, target_count // 5))
    noisy_count = target_count - standard_count
    for index in range(standard_count):
        count = PEOPLE_COUNTS[index % len(PEOPLE_COUNTS)]
        date = DATES[index % len(DATES)]
        time = TIMES[index % len(TIMES)]
        name = NAMES[index % len(NAMES)]
        examples.append(templates[index % len(templates)](count, date, time, name))
    for index in range(noisy_count):
        count = PEOPLE_COUNTS[(index + 2) % len(PEOPLE_COUNTS)]
        date = DATES[(index + 3) % len(DATES)]
        time = TIMES[(index + 4) % len(TIMES)]
        template_index = index % len(noisy_templates)
        if template_index in {0, 2, 4}:
            contact = PHONES[index % len(PHONES)]
        else:
            contact = EMAILS[index % len(EMAILS)]
        examples.append(noisy_templates[template_index](count, date, time, contact))
    return examples[:target_count]


def reservation_modify_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda n, d, t: build_example(
            "reservation_modify",
            ["Please move my existing reservation for ", ("PERSON", n), " to ", ("DATE", d), " at ", ("TIME", t)],
        ),
        lambda n, c, t: build_example(
            "reservation_modify",
            ["Change the booking I already have for ", ("PERSON", n), " to ", ("PEOPLE_COUNT", c), " people at ", ("TIME", t)],
        ),
        lambda p, d, t: build_example(
            "reservation_modify",
            ["I already booked a table, and I need to update my current reservation to ", ("DATE", d), " at ", ("TIME", t), ", phone ", ("PHONE", p)],
        ),
        lambda e, c, d: build_example(
            "reservation_modify",
            ["Modify the reservation I already made tied to ", ("EMAIL", e), " to ", ("PEOPLE_COUNT", c), " guests on ", ("DATE", d)],
        ),
        lambda n, d, c: build_example(
            "reservation_modify",
            ["For ", ("PERSON", n), ", reschedule the existing booking I already made to ", ("DATE", d), " for ", ("PEOPLE_COUNT", c)],
        ),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        name = pick(NAMES, bucket, template_index * 2, step=3)
        count = pick(PEOPLE_COUNTS, bucket, template_index, step=2)
        date = pick(DATES, bucket, template_index * 3, step=3)
        time = pick(TIMES, bucket, template_index * 4, step=4)
        phone = pick(PHONES, bucket, template_index * 5, step=5)
        email = pick(EMAILS, bucket, template_index * 6, step=6)
        choices = [
            templates[0](name, date, time),
            templates[1](name, count, time),
            templates[2](phone, date, time),
            templates[3](email, count, date),
            templates[4](name, date, count),
        ]
        examples.append(choices[template_index])
    return examples[:target_count]


def reservation_cancel_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda n, d: build_example(
            "reservation_cancel",
            ["Cancel the reservation for ", ("PERSON", n), " on ", ("DATE", d)],
        ),
        lambda p, t: build_example(
            "reservation_cancel",
            ["Please cancel my booking at ", ("TIME", t), ", phone ", ("PHONE", p)],
        ),
        lambda e, d: build_example(
            "reservation_cancel",
            ["Drop the reservation linked to ", ("EMAIL", e), " for ", ("DATE", d)],
        ),
        lambda n, t: build_example(
            "reservation_cancel",
            ["I need to cancel ", ("PERSON", n), "'s table at ", ("TIME", t)],
        ),
        lambda p, d: build_example(
            "reservation_cancel",
            ["Can you remove my reservation for ", ("DATE", d), "? my number is ", ("PHONE", p)],
        ),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        name = pick(NAMES, bucket, template_index, step=2)
        date = pick(DATES, bucket, template_index * 2, step=3)
        time = pick(TIMES, bucket, template_index * 3, step=4)
        phone = pick(PHONES, bucket, template_index * 4, step=5)
        email = pick(EMAILS, bucket, template_index * 5, step=6)
        choices = [
            templates[0](name, date),
            templates[1](phone, time),
            templates[2](email, date),
            templates[3](name, time),
            templates[4](phone, date),
        ]
        examples.append(choices[template_index])
    return examples[:target_count]


def reservation_status_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda n, context: build_example("reservation_status", ["Can you check my reservation under ", ("PERSON", n), " ", context, "?"]),
        lambda d, context: build_example("reservation_status", ["Please show my reservation for ", ("DATE", d), " ", context]),
        lambda t, context: build_example("reservation_status", ["I want to recheck my reservation at ", ("TIME", t), " ", context]),
        lambda p, context: build_example("reservation_status", ["Can you view the reservation linked to ", ("PHONE", p), " ", context, "?"]),
        lambda e, context: build_example("reservation_status", ["What is the status of my reservation for ", ("EMAIL", e), " ", context, "?"]),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        name = pick(NAMES, bucket, template_index * 2, step=3)
        date = pick(DATES, bucket, template_index * 3, step=4)
        time = pick(TIMES, bucket, template_index * 4, step=5)
        phone = pick(PHONES, bucket, template_index * 5, step=6)
        email = pick(EMAILS, bucket, template_index * 6, step=7)
        context = pick(RESERVATION_STATUS_CONTEXTS, bucket, template_index * 7, step=2)
        choices = [
            templates[0](name, context),
            templates[1](date, context),
            templates[2](time, context),
            templates[3](phone, context),
            templates[4](email, context),
        ]
        examples.append(choices[template_index])
    return examples[:target_count]


def menu_request_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda topic, context: build_example("menu_request", ["Can I see the ", ("MENU_ITEM", topic), " ", context, "?"]),
        lambda topic, context: build_example("menu_request", ["Do you have ", ("MENU_ITEM", topic), " on the menu ", context, "?"]),
        lambda topic, context: build_example("menu_request", ["Please send me your ", ("MENU_ITEM", topic), " ", context]),
        lambda topic, context: build_example("menu_request", ["What does the ", ("MENU_ITEM", topic), " look like ", context, "?"]),
        lambda topic, context: build_example("menu_request", ["I'm checking the ", ("MENU_ITEM", topic), " ", context]),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        topic = pick(MENU_TOPICS, bucket, template_index * 2, step=3)
        context = pick(MENU_CONTEXTS, bucket, template_index * 3, step=4)
        examples.append(templates[template_index](topic, context))
    return examples[:target_count]


def opening_hours_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda d, t, context: build_example(
            "opening_hours",
            ["Are you open on ", ("DATE", d), " at ", ("TIME", t), " ", context, "?"],
        ),
        lambda d, context: build_example("opening_hours", ["What time do you open on ", ("DATE", d), " ", context, "?"]),
        lambda d, context: build_example("opening_hours", ["What time do you close on ", ("DATE", d), " ", context, "?"]),
        lambda d, t, context: build_example(
            "opening_hours",
            ["Will the restaurant still be serving dinner ", ("DATE", d), " around ", ("TIME", t), " ", context, "?"],
        ),
        lambda d, context: build_example("opening_hours", ["Are your hours different ", ("DATE", d), " ", context, "?"]),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        date = pick(DAY_PHRASES, bucket, template_index * 2, step=3)
        time = pick(TIMES, bucket, template_index * 3, step=4)
        context = pick(HOUR_CONTEXTS, bucket, template_index * 4, step=5)
        choices = [
            templates[0](date, time, context),
            templates[1](date, context),
            templates[2](date, context),
            templates[3](date, time, context),
            templates[4](date, context),
        ]
        examples.append(choices[template_index])
    return examples[:target_count]


def location_request_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda location, context: build_example("location_request", ["Are you near ", ("LOCATION", location), "? ", context, "."]),
        lambda location, context: build_example("location_request", ["What is your address by ", ("LOCATION", location), "? ", context, "."]),
        lambda location, context: build_example("location_request", ["How do I get to the restaurant from ", ("LOCATION", location), "? ", context, "."]),
        lambda location, context: build_example("location_request", ["Is there parking close to ", ("LOCATION", location), "? ", context, "."]),
        lambda location, context: build_example("location_request", ["Which part of town are you in near ", ("LOCATION", location), "? ", context, "."]),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        location = pick(LOCATIONS, bucket, template_index * 2, step=3)
        context = pick(LOCATION_CONTEXTS, bucket, template_index * 3, step=4)
        examples.append(templates[template_index](location, context))
    return examples[:target_count]


def pricing_request_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda topic, context: build_example("pricing_request", ["How much is the ", ("PRICE_ITEM", topic), " ", context, "?"]),
        lambda topic, context: build_example("pricing_request", ["What's the price range for the ", ("PRICE_ITEM", topic), " ", context, "?"]),
        lambda topic, context: build_example("pricing_request", ["Can you tell me the cost of the ", ("PRICE_ITEM", topic), " ", context, "?"]),
        lambda topic, context: build_example("pricing_request", ["Is the ", ("PRICE_ITEM", topic), " expensive ", context, "?"]),
        lambda topic, context: build_example("pricing_request", ["I need the price for the ", ("PRICE_ITEM", topic), " ", context]),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        topic = pick(PRICE_TOPICS, bucket, template_index * 2, step=3)
        context = pick(PRICE_CONTEXTS, bucket, template_index * 3, step=4)
        examples.append(templates[template_index](topic, context))
    return examples[:target_count]


def contact_request_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda p, context, suffix: build_example("contact_request", ["What is the best phone number to reach you ", context, " ", suffix, "? I have ", ("PHONE", p), " in my notes"]),
        lambda e, context, suffix: build_example("contact_request", ["Is ", ("EMAIL", e), " the right email ", context, " ", suffix, "?"]),
        lambda n, context, suffix: build_example("contact_request", ["Can ", ("PERSON", n), " call the restaurant manager ", context, " ", suffix, "?"]),
        lambda p, context, suffix: build_example("contact_request", ["Should I text ", ("PHONE", p), " or call ", context, " ", suffix, "?"]),
        lambda e, context, suffix: build_example("contact_request", ["Can I contact the team at ", ("EMAIL", e), " ", context, " ", suffix, "?"]),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        phone = pick(PHONES, bucket, template_index * 2, step=3)
        email = pick(EMAILS, bucket, template_index * 3, step=4)
        name = pick(NAMES, bucket, template_index * 4, step=5)
        context = pick(CONTACT_CONTEXTS, bucket, template_index * 5, step=6)
        suffix = pick(CONTACT_SUFFIXES, bucket, template_index * 7, step=2)
        choices = [
            templates[0](phone, context, suffix),
            templates[1](email, context, suffix),
            templates[2](name, context, suffix),
            templates[3](phone, context, suffix),
            templates[4](email, context, suffix),
        ]
        examples.append(choices[template_index])
    return examples[:target_count]


def unknown_examples(target_count: int) -> list[Example]:
    examples: list[Example] = []
    templates = [
        lambda token, _: Example(text=token, intent="unknown", entities=tuple()),
        lambda phrase, _: Example(text=phrase, intent="unknown", entities=tuple()),
        lambda phrase, suffix: Example(text=f"{phrase} {suffix}", intent="unknown", entities=tuple()),
        lambda phrase, _: Example(text=f"{phrase} please", intent="unknown", entities=tuple()),
    ]
    for index in range(target_count):
        template_index = index % len(templates)
        bucket = index // len(templates)
        token = pick(UNKNOWN_SINGLE_TOKENS, bucket, template_index * 2, step=3)
        phrase = pick(UNKNOWN_SMALL_TALK, bucket, template_index * 3, step=4)
        vague = pick(UNKNOWN_VAGUE_FOLLOW_UPS, bucket, template_index * 4, step=5)
        suffix = pick(UNKNOWN_SINGLE_TOKENS, bucket, template_index * 5, step=2)
        choices = [
            templates[0](token, suffix),
            templates[1](phrase, suffix),
            templates[2](vague, suffix),
            templates[3](phrase, suffix),
        ]
        examples.append(choices[template_index])
    return examples[:target_count]


def build_corpus() -> list[Example]:
    corpus = []
    corpus.extend(reservation_create_examples(INTENT_TARGETS["reservation_create"]))
    corpus.extend(reservation_modify_examples(INTENT_TARGETS["reservation_modify"]))
    corpus.extend(reservation_cancel_examples(INTENT_TARGETS["reservation_cancel"]))
    corpus.extend(reservation_status_examples(INTENT_TARGETS["reservation_status"]))
    corpus.extend(menu_request_examples(INTENT_TARGETS["menu_request"]))
    corpus.extend(opening_hours_examples(INTENT_TARGETS["opening_hours"]))
    corpus.extend(location_request_examples(INTENT_TARGETS["location_request"]))
    corpus.extend(pricing_request_examples(INTENT_TARGETS["pricing_request"]))
    corpus.extend(contact_request_examples(INTENT_TARGETS["contact_request"]))
    corpus.extend(unknown_examples(INTENT_TARGETS["unknown"]))
    return ensure_unique_texts(corpus)


def ensure_unique_texts(examples: Sequence[Example]) -> list[Example]:
    """Ensure every utterance text is unique while keeping entity spans valid."""

    deduped: list[Example] = []
    seen_counts: dict[str, int] = {}
    for example in examples:
        duplicate_index = seen_counts.get(example.text, 0)
        if duplicate_index == 0:
            deduped.append(example)
        else:
            suffix = UNIQUENESS_SUFFIXES[(duplicate_index - 1) % len(UNIQUENESS_SUFFIXES)]
            cycle = duplicate_index // len(UNIQUENESS_SUFFIXES)
            if cycle > 0:
                suffix = f"{suffix} variation {cycle + 1}"
            deduped.append(
                Example(
                    text=f"{example.text}{suffix}",
                    intent=example.intent,
                    entities=example.entities,
                )
            )
        seen_counts[example.text] = duplicate_index + 1
    return deduped


def validate_examples(examples: Sequence[Example]) -> None:
    seen_texts: set[str] = set()
    for example in examples:
        if not example.intent:
            raise ValueError("empty intent label")
        if example.text in seen_texts:
            raise ValueError(f"duplicate text: {example.text}")
        seen_texts.add(example.text)
        for entity in example.entities:
            if entity.start < 0 or entity.end <= entity.start or entity.end > len(example.text):
                raise ValueError(f"invalid span in example: {example.text}")
            extracted = example.text[entity.start : entity.end]
            if not extracted:
                raise ValueError(f"empty span in example: {example.text}")


def write_jsonl(path: Path, examples: Sequence[Example]) -> None:
    with path.open("w", encoding="utf-8", newline="\n") as handle:
        for example in examples:
            handle.write(json.dumps(example.to_payload(), ensure_ascii=True) + "\n")


def split_examples(corpus: Sequence[Example]) -> tuple[list[Example], list[Example], list[Example]]:
    split_counts: dict[str, tuple[int, int, int]] = {
        "reservation_create": (608, 76, 76),
        "reservation_modify": (432, 54, 54),
        "reservation_cancel": (304, 38, 38),
        "reservation_status": (208, 26, 26),
        "menu_request": (512, 64, 64),
        "opening_hours": (448, 56, 56),
        "location_request": (432, 54, 54),
        "pricing_request": (432, 54, 54),
        "contact_request": (624, 78, 78),
        "unknown": (400, 50, 50),
    }
    train: list[Example] = []
    validation: list[Example] = []
    evaluation: list[Example] = []
    grouped: dict[str, list[Example]] = {}
    for example in corpus:
        grouped.setdefault(example.intent, []).append(example)
    for intent, (train_count, validation_count, eval_count) in split_counts.items():
        intent_examples = grouped[intent]
        expected_total = train_count + validation_count + eval_count
        if len(intent_examples) != expected_total:
            raise ValueError(f"intent {intent} expected {expected_total} examples, found {len(intent_examples)}")
        train.extend(intent_examples[:train_count])
        validation.extend(intent_examples[train_count : train_count + validation_count])
        evaluation.extend(intent_examples[train_count + validation_count : expected_total])
    return train, validation, evaluation


def main() -> None:
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    corpus = build_corpus()
    validate_examples(corpus)
    if len(corpus) != EXPECTED_CORPUS_SIZE:
        raise ValueError(f"expected {EXPECTED_CORPUS_SIZE} examples, found {len(corpus)}")

    train, validation, evaluation = split_examples(corpus)
    for split_name, split_examples_list, expected_size in (
        ("train", train, EXPECTED_SPLIT_SIZES["train"]),
        ("validation", validation, EXPECTED_SPLIT_SIZES["validation"]),
        ("eval", evaluation, EXPECTED_SPLIT_SIZES["eval"]),
    ):
        validate_examples(split_examples_list)
        if len(split_examples_list) != expected_size:
            raise ValueError(f"{split_name} expected {expected_size} examples, found {len(split_examples_list)}")

    write_jsonl(OUTPUT_DIR / "restaurant_corpus.jsonl", corpus)
    write_jsonl(OUTPUT_DIR / "restaurant_train.jsonl", train)
    write_jsonl(OUTPUT_DIR / "restaurant_validation.jsonl", validation)
    write_jsonl(OUTPUT_DIR / "restaurant_eval.jsonl", evaluation)


if __name__ == "__main__":
    main()
