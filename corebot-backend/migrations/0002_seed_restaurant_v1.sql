insert into businesses (id, name, domain_type, default_locale)
values ('11111111-1111-1111-1111-111111111111', 'Corebot Restaurant', 'restaurant', 'en');

insert into business_locations (id, business_id, label, address_line, city, postal_code, country, nearby_description)
values ('11111111-1111-1111-1111-111111111201', '11111111-1111-1111-1111-111111111111', 'main', '12 Rue de la Paix, 75001 Paris', 'Paris', '75001', 'FR', 'near the city center, by the river');

insert into contact_channels (id, business_id, channel_type, value, label, is_primary) values
('11111111-1111-1111-1111-111111111301', '11111111-1111-1111-1111-111111111111', 'phone', '+33123456789', 'Booking phone', true),
('11111111-1111-1111-1111-111111111302', '11111111-1111-1111-1111-111111111111', 'email', 'booking@example.com', 'Booking email', true),
('11111111-1111-1111-1111-111111111303', '11111111-1111-1111-1111-111111111111', 'events_email', 'events@example.com', 'Events email', false);

insert into restaurant_profiles (business_id, takeaway_available, delivery_available)
values ('11111111-1111-1111-1111-111111111111', true, true);

insert into restaurant_opening_hours (id, business_id, day_of_week, opens_at, closes_at) values
('11111111-1111-1111-1111-111111111401', '11111111-1111-1111-1111-111111111111', 1, '11:00', '22:00'),
('11111111-1111-1111-1111-111111111402', '11111111-1111-1111-1111-111111111111', 2, '11:00', '22:00'),
('11111111-1111-1111-1111-111111111403', '11111111-1111-1111-1111-111111111111', 3, '11:00', '22:00'),
('11111111-1111-1111-1111-111111111404', '11111111-1111-1111-1111-111111111111', 4, '11:00', '22:00'),
('11111111-1111-1111-1111-111111111405', '11111111-1111-1111-1111-111111111111', 5, '11:00', '22:00'),
('11111111-1111-1111-1111-111111111406', '11111111-1111-1111-1111-111111111111', 6, '11:00', '22:00'),
('11111111-1111-1111-1111-111111111407', '11111111-1111-1111-1111-111111111111', 7, '11:00', '22:00');

insert into restaurant_reservation_settings (business_id, slot_minutes, max_lookup_days)
values ('11111111-1111-1111-1111-111111111111', 120, 7);

insert into restaurant_table_types (id, business_id, capacity, table_count) values
('11111111-1111-1111-1111-111111111501', '11111111-1111-1111-1111-111111111111', 6, 2),
('11111111-1111-1111-1111-111111111502', '11111111-1111-1111-1111-111111111111', 4, 3),
('11111111-1111-1111-1111-111111111503', '11111111-1111-1111-1111-111111111111', 2, 3);

insert into menus (id, business_id, code, active)
values ('11111111-1111-1111-1111-111111111601', '11111111-1111-1111-1111-111111111111', 'main', true);
insert into menu_translations (menu_id, locale, name, description)
values ('11111111-1111-1111-1111-111111111601', 'en', 'Main menu', 'Full restaurant menu');
insert into menu_sections (id, menu_id, sort_order)
values ('11111111-1111-1111-1111-111111111602', '11111111-1111-1111-1111-111111111601', 1);
insert into menu_section_translations (section_id, locale, name)
values ('11111111-1111-1111-1111-111111111602', 'en', 'A la carte');

insert into dietary_tags (id, code) values
('11111111-1111-1111-1111-111111111701', 'vegan'),
('11111111-1111-1111-1111-111111111702', 'vegetarian'),
('11111111-1111-1111-1111-111111111703', 'halal'),
('11111111-1111-1111-1111-111111111704', 'gluten-free'),
('11111111-1111-1111-1111-111111111705', 'dairy-free'),
('11111111-1111-1111-1111-111111111706', 'nut-free');

insert into allergen_tags (id, code) values
('11111111-1111-1111-1111-111111111801', 'gluten'),
('11111111-1111-1111-1111-111111111802', 'dairy'),
('11111111-1111-1111-1111-111111111803', 'eggs'),
('11111111-1111-1111-1111-111111111804', 'soy'),
('11111111-1111-1111-1111-111111111805', 'shellfish'),
('11111111-1111-1111-1111-111111111806', 'sesame'),
('11111111-1111-1111-1111-111111111807', 'peanuts');

insert into menu_items (id, business_id, code, price_cents, currency) values
('11111111-1111-1111-1111-111111112001', '11111111-1111-1111-1111-111111111111', 'pizza', 1200, 'EUR'),
('11111111-1111-1111-1111-111111112002', '11111111-1111-1111-1111-111111111111', 'salad', 800, 'EUR'),
('11111111-1111-1111-1111-111111112003', '11111111-1111-1111-1111-111111111111', 'chocolate_cake', 600, 'EUR'),
('11111111-1111-1111-1111-111111112004', '11111111-1111-1111-1111-111111111111', 'fried_rice', 1000, 'EUR'),
('11111111-1111-1111-1111-111111112005', '11111111-1111-1111-1111-111111111111', 'vegetarian_pasta', 1100, 'EUR'),
('11111111-1111-1111-1111-111111112006', '11111111-1111-1111-1111-111111111111', 'seafood_soup', 1400, 'EUR'),
('11111111-1111-1111-1111-111111112007', '11111111-1111-1111-1111-111111111111', 'beef_burger', 1400, 'EUR'),
('11111111-1111-1111-1111-111111112008', '11111111-1111-1111-1111-111111111111', 'chicken_satay', 1300, 'EUR'),
('11111111-1111-1111-1111-111111112009', '11111111-1111-1111-1111-111111111111', 'vegan_curry', 1100, 'EUR'),
('11111111-1111-1111-1111-111111112010', '11111111-1111-1111-1111-111111111111', 'kids_pasta', 800, 'EUR'),
('11111111-1111-1111-1111-111111112011', '11111111-1111-1111-1111-111111111111', 'set_menu', 3500, 'EUR'),
('11111111-1111-1111-1111-111111112012', '11111111-1111-1111-1111-111111111111', 'lunch_special', 1500, 'EUR'),
('11111111-1111-1111-1111-111111112013', '11111111-1111-1111-1111-111111111111', 'kids_menu', 1000, 'EUR'),
('11111111-1111-1111-1111-111111112014', '11111111-1111-1111-1111-111111111111', 'breakfast_menu', 1200, 'EUR'),
('11111111-1111-1111-1111-111111112015', '11111111-1111-1111-1111-111111111111', 'family_menu', 6000, 'EUR'),
('11111111-1111-1111-1111-111111112016', '11111111-1111-1111-1111-111111111111', 'tasting_menu', 7500, 'EUR'),
('11111111-1111-1111-1111-111111112017', '11111111-1111-1111-1111-111111111111', 'dessert_menu', 1800, 'EUR');

insert into menu_item_translations (menu_item_id, locale, name) values
('11111111-1111-1111-1111-111111112001', 'en', 'pizza'),
('11111111-1111-1111-1111-111111112002', 'en', 'salad'),
('11111111-1111-1111-1111-111111112003', 'en', 'chocolate cake'),
('11111111-1111-1111-1111-111111112004', 'en', 'fried rice'),
('11111111-1111-1111-1111-111111112005', 'en', 'vegetarian pasta'),
('11111111-1111-1111-1111-111111112006', 'en', 'seafood soup'),
('11111111-1111-1111-1111-111111112007', 'en', 'beef burger'),
('11111111-1111-1111-1111-111111112008', 'en', 'chicken satay'),
('11111111-1111-1111-1111-111111112009', 'en', 'vegan curry'),
('11111111-1111-1111-1111-111111112010', 'en', 'kids pasta'),
('11111111-1111-1111-1111-111111112011', 'en', 'set menu'),
('11111111-1111-1111-1111-111111112012', 'en', 'lunch special'),
('11111111-1111-1111-1111-111111112013', 'en', 'kids menu'),
('11111111-1111-1111-1111-111111112014', 'en', 'breakfast menu'),
('11111111-1111-1111-1111-111111112015', 'en', 'family menu'),
('11111111-1111-1111-1111-111111112016', 'en', 'tasting menu'),
('11111111-1111-1111-1111-111111112017', 'en', 'dessert menu');

insert into menu_section_items (section_id, menu_item_id, sort_order)
select '11111111-1111-1111-1111-111111111602', id, row_number() over (order by code)
from menu_items
where business_id = '11111111-1111-1111-1111-111111111111';

insert into menu_item_dietary_tags (menu_item_id, dietary_tag_id)
select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code in ('vegetarian') where mi.code in ('pizza','chocolate_cake','vegetarian_pasta','kids_pasta','kids_menu','breakfast_menu','dessert_menu')
union all select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code in ('vegan','vegetarian','gluten-free','dairy-free','nut-free') where mi.code = 'salad'
union all select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code in ('gluten-free') where mi.code = 'fried_rice'
union all select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code in ('gluten-free','dairy-free') where mi.code = 'seafood_soup'
union all select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code in ('halal','gluten-free','dairy-free') where mi.code = 'chicken_satay'
union all select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code in ('vegan','vegetarian','halal','gluten-free','dairy-free') where mi.code = 'vegan_curry';

insert into menu_item_allergen_tags (menu_item_id, allergen_tag_id)
select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten','dairy') where mi.code in ('pizza','set_menu','family_menu')
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten','dairy','eggs') where mi.code in ('chocolate_cake','vegetarian_pasta','kids_pasta','kids_menu','breakfast_menu','dessert_menu')
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('eggs','soy') where mi.code = 'fried_rice'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('shellfish','soy') where mi.code = 'seafood_soup'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten','dairy','eggs','sesame') where mi.code = 'beef_burger'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('peanuts','soy') where mi.code = 'chicken_satay'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('soy') where mi.code = 'vegan_curry'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten') where mi.code = 'lunch_special'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten','dairy','shellfish') where mi.code = 'tasting_menu';

insert into business_payment_methods (business_id, method_code) values
('11111111-1111-1111-1111-111111111111', 'credit card'),
('11111111-1111-1111-1111-111111111111', 'cash'),
('11111111-1111-1111-1111-111111111111', 'Apple Pay'),
('11111111-1111-1111-1111-111111111111', 'Google Pay'),
('11111111-1111-1111-1111-111111111111', 'Visa'),
('11111111-1111-1111-1111-111111111111', 'Mastercard'),
('11111111-1111-1111-1111-111111111111', 'contactless');

insert into business_facilities (id, business_id, facility_code, label) values
('11111111-1111-1111-1111-111111113001', '11111111-1111-1111-1111-111111111111', 'baby_seat', 'baby seat'),
('11111111-1111-1111-1111-111111113002', '11111111-1111-1111-1111-111111111111', 'parking', 'parking'),
('11111111-1111-1111-1111-111111113003', '11111111-1111-1111-1111-111111111111', 'wifi', 'wifi'),
('11111111-1111-1111-1111-111111113004', '11111111-1111-1111-1111-111111111111', 'high_chairs', 'high chairs'),
('11111111-1111-1111-1111-111111113005', '11111111-1111-1111-1111-111111111111', 'outdoor_seating', 'outdoor seating'),
('11111111-1111-1111-1111-111111113006', '11111111-1111-1111-1111-111111111111', 'private_room', 'private room'),
('11111111-1111-1111-1111-111111113007', '11111111-1111-1111-1111-111111111111', 'bike_parking', 'bike parking');

insert into business_facts (id, business_id, fact_type, metadata) values
('11111111-1111-1111-1111-111111114001', '11111111-1111-1111-1111-111111111111', 'takeaway', '{"delivery": true}'),
('11111111-1111-1111-1111-111111114002', '11111111-1111-1111-1111-111111111111', 'accessibility', '{}'),
('11111111-1111-1111-1111-111111114003', '11111111-1111-1111-1111-111111111111', 'entertainment', '{}');

insert into business_fact_translations (fact_id, locale, title, content) values
('11111111-1111-1111-1111-111111114001', 'en', 'Takeaway', 'We offer takeaway and delivery. Order by phone or at the counter.'),
('11111111-1111-1111-1111-111111114002', 'en', 'Accessibility', 'The restaurant is wheelchair accessible with step-free access at the main entrance. Strollers are welcome.'),
('11111111-1111-1111-1111-111111114003', 'en', 'Entertainment', 'We have live music every Friday and Saturday evening. A DJ performs on Saturday nights.');

insert into restaurant_event_spaces (id, business_id, name, description, contact_channel_id) values
('11111111-1111-1111-1111-111111115001', '11111111-1111-1111-1111-111111111111', 'terrace', 'Available for birthday parties, corporate events, and private dinners.', '11111111-1111-1111-1111-111111111303'),
('11111111-1111-1111-1111-111111115002', '11111111-1111-1111-1111-111111111111', 'private room', 'Available for birthday parties, corporate events, and private dinners.', '11111111-1111-1111-1111-111111111303');

insert into reservations (id, business_id, reference, customer_name, reservation_date, reservation_time, people_count, status) values
('11111111-1111-1111-1111-111111116001', '11111111-1111-1111-1111-111111111111', 'REST-ABC123', 'Maya Chen', '2026-08-23', '19:00', 2, 'confirmed'),
('11111111-1111-1111-1111-111111116002', '11111111-1111-1111-1111-111111111111', 'REST-ZX90K2', 'Jean Martin', '2026-06-12', '20:00', 4, 'confirmed'),
('11111111-1111-1111-1111-111111116003', '11111111-1111-1111-1111-111111111111', 'REST-2026A1', 'Priya Singh', '2026-07-08', '19:30', 3, 'confirmed'),
('11111111-1111-1111-1111-111111116004', '11111111-1111-1111-1111-111111111111', 'REST-7F4K2A', 'Noah Davis', '2026-05-20', '18:45', 6, 'confirmed'),
('11111111-1111-1111-1111-111111116005', '11111111-1111-1111-1111-111111111111', 'REST-MN45QP', 'Alice Brown', '2026-09-15', '12:00', 2, 'confirmed'),
('11111111-1111-1111-1111-111111116006', '11111111-1111-1111-1111-111111111111', 'REST-9X8Y7Z', 'Sam Wilson', '2026-08-01', '21:00', 5, 'confirmed'),
('11111111-1111-1111-1111-111111116007', '11111111-1111-1111-1111-111111111111', 'REST-BOOK42', 'Omar Khan', '2026-06-30', '19:15', 8, 'confirmed'),
('11111111-1111-1111-1111-111111116008', '11111111-1111-1111-1111-111111111111', 'REST-CXL777', 'Lena Smith', '2026-07-25', '18:00', 1, 'confirmed'),
('11111111-1111-1111-1111-111111116009', '11111111-1111-1111-1111-111111111111', 'REST-A1B2C3', 'Alex Carter', '2026-10-03', '20:30', 10, 'confirmed'),
('11111111-1111-1111-1111-111111116010', '11111111-1111-1111-1111-111111111111', 'REST-TABLE9', 'Nina Patel', '2026-11-12', '13:00', 4, 'confirmed');
