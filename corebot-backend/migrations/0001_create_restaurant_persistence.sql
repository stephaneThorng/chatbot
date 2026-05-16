create table businesses (
    id uuid primary key,
    name text not null,
    domain_type text not null,
    default_locale text not null default 'en',
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create table business_locations (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    label text,
    address_line text not null,
    city text,
    postal_code text,
    country text,
    latitude numeric,
    longitude numeric,
    nearby_description text
);

create table contact_channels (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    channel_type text not null,
    value text not null,
    label text,
    is_primary boolean not null default false,
    active boolean not null default true
);

create table restaurant_profiles (
    business_id uuid primary key references businesses(id),
    takeaway_available boolean not null default false,
    delivery_available boolean not null default false,
    accessibility_description text,
    entertainment_description text
);

create table restaurant_opening_hours (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    day_of_week smallint not null check (day_of_week between 1 and 7),
    opens_at time not null,
    closes_at time not null,
    is_closed boolean not null default false,
    unique (business_id, day_of_week)
);

create table business_closures (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    starts_at timestamp not null,
    ends_at timestamp not null,
    reason text,
    closure_type text not null,
    check (ends_at > starts_at)
);

create table restaurant_reservation_settings (
    business_id uuid primary key references businesses(id),
    slot_minutes integer not null check (slot_minutes > 0),
    max_lookup_days integer not null default 7 check (max_lookup_days > 0)
);

create table restaurant_table_types (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    capacity integer not null check (capacity > 0),
    table_count integer not null check (table_count >= 0)
);

create table menus (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    code text not null,
    active boolean not null default true,
    unique (business_id, code)
);

create table menu_translations (
    menu_id uuid not null references menus(id),
    locale text not null,
    name text not null,
    description text,
    primary key (menu_id, locale)
);

create table menu_sections (
    id uuid primary key,
    menu_id uuid not null references menus(id),
    sort_order integer not null
);

create table menu_section_translations (
    section_id uuid not null references menu_sections(id),
    locale text not null,
    name text not null,
    primary key (section_id, locale)
);

create table menu_items (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    code text not null,
    price_cents integer not null check (price_cents >= 0),
    currency char(3) not null default 'EUR',
    active boolean not null default true,
    unique (business_id, code)
);

create table menu_item_translations (
    menu_item_id uuid not null references menu_items(id),
    locale text not null,
    name text not null,
    description text,
    primary key (menu_item_id, locale)
);

create table menu_section_items (
    section_id uuid not null references menu_sections(id),
    menu_item_id uuid not null references menu_items(id),
    sort_order integer not null,
    price_cents_override integer,
    primary key (section_id, menu_item_id)
);

create table dietary_tags (
    id uuid primary key,
    code text unique not null
);

create table allergen_tags (
    id uuid primary key,
    code text unique not null
);

create table menu_item_dietary_tags (
    menu_item_id uuid not null references menu_items(id),
    dietary_tag_id uuid not null references dietary_tags(id),
    primary key (menu_item_id, dietary_tag_id)
);

create table menu_item_allergen_tags (
    menu_item_id uuid not null references menu_items(id),
    allergen_tag_id uuid not null references allergen_tags(id),
    primary key (menu_item_id, allergen_tag_id)
);

create table business_facts (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    fact_type text not null,
    metadata jsonb not null default '{}',
    active boolean not null default true
);

create table business_fact_translations (
    fact_id uuid not null references business_facts(id),
    locale text not null,
    title text,
    content text not null,
    primary key (fact_id, locale)
);

create table business_facilities (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    facility_code text not null,
    label text not null,
    unique (business_id, facility_code)
);

create table business_payment_methods (
    business_id uuid not null references businesses(id),
    method_code text not null,
    primary key (business_id, method_code)
);

create table restaurant_event_spaces (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    name text not null,
    description text,
    contact_channel_id uuid references contact_channels(id)
);

create table reservations (
    id uuid primary key,
    business_id uuid not null references businesses(id),
    reference text not null unique,
    customer_name text not null,
    reservation_date date not null,
    reservation_time time not null,
    people_count integer not null check (people_count > 0),
    status text not null,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);

create index reservations_business_slot_idx
    on reservations (business_id, reservation_date, reservation_time, status);
create index reservations_reference_lookup_idx
    on reservations (business_id, lower(reference));
create index reservations_customer_name_idx
    on reservations (business_id, lower(customer_name));
create index business_closures_range_idx
    on business_closures (business_id, starts_at, ends_at);
