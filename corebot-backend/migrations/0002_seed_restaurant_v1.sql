insert into businesses (id, name, domain_type, default_locale)
values ('11111111-1111-1111-1111-111111111111', 'Koru Kulture & Koffee', 'restaurant', 'en');

insert into business_locations (id, business_id, label, address_line, city, postal_code, country, nearby_description)
values (
    '11111111-1111-1111-1111-111111111201',
    '11111111-1111-1111-1111-111111111111',
    'main',
    'Jl. Batur Sari No.43, Sanur, Denpasar Selatan, Kota Denpasar, Bali 80228',
    'Denpasar',
    '80228',
    'ID',
    'in Sanur, Bali'
);

insert into contact_channels (id, business_id, channel_type, value, label, is_primary) values
('11111111-1111-1111-1111-111111111301', '11111111-1111-1111-1111-111111111111', 'phone', '+628881112222', 'Restaurant phone', true),
('11111111-1111-1111-1111-111111111302', '11111111-1111-1111-1111-111111111111', 'email', 'hello@korukulture.example.com', 'Restaurant email', true),
('11111111-1111-1111-1111-111111111303', '11111111-1111-1111-1111-111111111111', 'events_email', 'manager@korukulture.example.com', 'Manager email', false);

insert into restaurant_profiles (business_id, takeaway_available, delivery_available)
values ('11111111-1111-1111-1111-111111111111', false, false);

insert into restaurant_opening_hours (id, business_id, day_of_week, opens_at, closes_at) values
('11111111-1111-1111-1111-111111111401', '11111111-1111-1111-1111-111111111111', 1, '08:00', '18:00'),
('11111111-1111-1111-1111-111111111402', '11111111-1111-1111-1111-111111111111', 2, '08:00', '18:00'),
('11111111-1111-1111-1111-111111111403', '11111111-1111-1111-1111-111111111111', 3, '08:00', '18:00'),
('11111111-1111-1111-1111-111111111404', '11111111-1111-1111-1111-111111111111', 4, '08:00', '18:00'),
('11111111-1111-1111-1111-111111111405', '11111111-1111-1111-1111-111111111111', 5, '08:00', '18:00'),
('11111111-1111-1111-1111-111111111406', '11111111-1111-1111-1111-111111111111', 6, '08:00', '18:00'),
('11111111-1111-1111-1111-111111111407', '11111111-1111-1111-1111-111111111111', 7, '08:00', '18:00');

insert into restaurant_reservation_settings (business_id, slot_minutes, max_lookup_days)
values ('11111111-1111-1111-1111-111111111111', 60, 7);

insert into restaurant_table_types (id, business_id, capacity, table_count) values
('11111111-1111-1111-1111-111111111501', '11111111-1111-1111-1111-111111111111', 6, 3);

insert into menus (id, business_id, code, active)
values ('11111111-1111-1111-1111-111111111601', '11111111-1111-1111-1111-111111111111', 'main', true);

insert into menu_translations (menu_id, locale, name, description)
values ('11111111-1111-1111-1111-111111111601', 'en', 'Main menu', 'Food and drinks menu');

insert into menu_sections (id, menu_id, sort_order) values
('11111111-1111-1111-1111-111111111602', '11111111-1111-1111-1111-111111111601', 1),
('11111111-1111-1111-1111-111111111603', '11111111-1111-1111-1111-111111111601', 2),
('11111111-1111-1111-1111-111111111604', '11111111-1111-1111-1111-111111111601', 3),
('11111111-1111-1111-1111-111111111605', '11111111-1111-1111-1111-111111111601', 4);

insert into menu_section_translations (section_id, locale, name) values
('11111111-1111-1111-1111-111111111602', 'en', 'Starters'),
('11111111-1111-1111-1111-111111111603', 'en', 'Main courses'),
('11111111-1111-1111-1111-111111111604', 'en', 'Desserts'),
('11111111-1111-1111-1111-111111111605', 'en', 'Drinks');

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
('11111111-1111-1111-1111-111111111807', 'peanuts'),
('11111111-1111-1111-1111-111111111808', 'fish'),
('11111111-1111-1111-1111-111111111809', 'tree nuts'),
('11111111-1111-1111-1111-111111111810', 'crustaceans');

insert into menu_items (id, business_id, code, price_cents, currency, active) values
('11111111-1111-1111-1111-111111112001', '11111111-1111-1111-1111-111111111111', 'chicken_spring_rolls', 4500000, 'IDR', true),
('11111111-1111-1111-1111-111111112002', '11111111-1111-1111-1111-111111111111', 'green_papaya_salad', 3500000, 'IDR', true),
('11111111-1111-1111-1111-111111112003', '11111111-1111-1111-1111-111111111111', 'chicken_satay_skewers', 5000000, 'IDR', true),
('11111111-1111-1111-1111-111111112004', '11111111-1111-1111-1111-111111111111', 'nasi_goreng_special', 6500000, 'IDR', true),
('11111111-1111-1111-1111-111111112005', '11111111-1111-1111-1111-111111111111', 'yellow_tofu_curry', 6000000, 'IDR', true),
('11111111-1111-1111-1111-111111112006', '11111111-1111-1111-1111-111111111111', 'bali_wagyu_burger', 12000000, 'IDR', true),
('11111111-1111-1111-1111-111111112007', '11111111-1111-1111-1111-111111111111', 'grilled_mahi_mahi', 9000000, 'IDR', false),
('11111111-1111-1111-1111-111111112008', '11111111-1111-1111-1111-111111111111', 'mango_sticky_rice', 4500000, 'IDR', true),
('11111111-1111-1111-1111-111111112009', '11111111-1111-1111-1111-111111111111', 'chocolate_lava_cake', 5500000, 'IDR', true),
('11111111-1111-1111-1111-111111112010', '11111111-1111-1111-1111-111111111111', 'homemade_kombucha', 3500000, 'IDR', true),
('11111111-1111-1111-1111-111111112011', '11111111-1111-1111-1111-111111111111', 'bintang_radler', 4000000, 'IDR', true);

insert into menu_item_translations (menu_item_id, locale, name, description) values
('11111111-1111-1111-1111-111111112001', 'en', 'chicken spring rolls', 'Traditional fried chicken and vegetable spring rolls, served with sweet and sour sauce.'),
('11111111-1111-1111-1111-111111112002', 'en', 'green papaya salad', 'Crisp green papaya, carrots, cherry tomatoes, lime dressing.'),
('11111111-1111-1111-1111-111111112003', 'en', 'chicken satay skewers', 'Wood-fired grilled chicken skewers, thick peanut sauce.'),
('11111111-1111-1111-1111-111111112004', 'en', 'nasi goreng special', 'Indonesian fried rice, fried egg, chicken, satay skewer, prawn crackers.'),
('11111111-1111-1111-1111-111111112005', 'en', 'yellow tofu curry', 'Mild coconut milk, local vegetables, fresh tofu, served with fragrant rice.'),
('11111111-1111-1111-1111-111111112006', 'en', 'bali wagyu burger', 'Wagyu beef patty, aged cheddar, brioche bun, spicy mayo, french fries.'),
('11111111-1111-1111-1111-111111112007', 'en', 'grilled mahi-mahi', 'Fresh morning-caught Mahi-Mahi fillet, sambal matah sauce, white rice.'),
('11111111-1111-1111-1111-111111112008', 'en', 'mango sticky rice', 'Warm sticky rice, fresh local mango, sweet coconut cream.'),
('11111111-1111-1111-1111-111111112009', 'en', 'chocolate lava cake', 'Dark chocolate cake with a molten center, vanilla ice cream scoop.'),
('11111111-1111-1111-1111-111111112010', 'en', 'homemade kombucha', 'Sparkling fermented tea, ginger and freshly squeezed lemon flavor.'),
('11111111-1111-1111-1111-111111112011', 'en', 'bintang radler', 'Light lager beer with lemon juice.');

insert into menu_section_items (section_id, menu_item_id, sort_order) values
('11111111-1111-1111-1111-111111111602', '11111111-1111-1111-1111-111111112001', 1),
('11111111-1111-1111-1111-111111111602', '11111111-1111-1111-1111-111111112002', 2),
('11111111-1111-1111-1111-111111111602', '11111111-1111-1111-1111-111111112003', 3),
('11111111-1111-1111-1111-111111111603', '11111111-1111-1111-1111-111111112004', 1),
('11111111-1111-1111-1111-111111111603', '11111111-1111-1111-1111-111111112005', 2),
('11111111-1111-1111-1111-111111111603', '11111111-1111-1111-1111-111111112006', 3),
('11111111-1111-1111-1111-111111111603', '11111111-1111-1111-1111-111111112007', 4),
('11111111-1111-1111-1111-111111111604', '11111111-1111-1111-1111-111111112008', 1),
('11111111-1111-1111-1111-111111111604', '11111111-1111-1111-1111-111111112009', 2),
('11111111-1111-1111-1111-111111111605', '11111111-1111-1111-1111-111111112010', 1),
('11111111-1111-1111-1111-111111111605', '11111111-1111-1111-1111-111111112011', 2);

insert into menu_item_dietary_tags (menu_item_id, dietary_tag_id)
select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code in ('vegan', 'gluten-free') where mi.code in ('green_papaya_salad', 'yellow_tofu_curry', 'mango_sticky_rice', 'homemade_kombucha')
union all select mi.id, dt.id from menu_items mi join dietary_tags dt on dt.code = 'gluten-free' where mi.code = 'grilled_mahi_mahi';

insert into menu_item_allergen_tags (menu_item_id, allergen_tag_id)
select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten', 'soy') where mi.code = 'chicken_spring_rolls'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('peanuts', 'soy') where mi.code = 'chicken_satay_skewers'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('crustaceans', 'eggs', 'gluten', 'peanuts') where mi.code = 'nasi_goreng_special'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('soy') where mi.code = 'yellow_tofu_curry'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten', 'dairy', 'eggs', 'sesame') where mi.code = 'bali_wagyu_burger'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('fish') where mi.code = 'grilled_mahi_mahi'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten', 'eggs', 'dairy', 'tree nuts') where mi.code = 'chocolate_lava_cake'
union all select mi.id, at.id from menu_items mi join allergen_tags at on at.code in ('gluten') where mi.code = 'bintang_radler';

insert into business_payment_methods (business_id, method_code) values
('11111111-1111-1111-1111-111111111111', 'cash'),
('11111111-1111-1111-1111-111111111111', 'Qris'),
('11111111-1111-1111-1111-111111111111', 'debit card'),
('11111111-1111-1111-1111-111111111111', 'credit card'),
('11111111-1111-1111-1111-111111111111', 'GoPay');

insert into business_facilities (id, business_id, facility_code, label) values
('11111111-1111-1111-1111-111111113001', '11111111-1111-1111-1111-111111111111', 'wifi', 'wifi'),
('11111111-1111-1111-1111-111111113002', '11111111-1111-1111-1111-111111111111', 'bike_parking', 'bike parking'),
('11111111-1111-1111-1111-111111113003', '11111111-1111-1111-1111-111111111111', 'parking', 'motorbike parking');

insert into business_facts (id, business_id, fact_type, metadata) values
('11111111-1111-1111-1111-111111114001', '11111111-1111-1111-1111-111111111111', 'takeaway', '{"delivery": false}'),
('11111111-1111-1111-1111-111111114002', '11111111-1111-1111-1111-111111111111', 'accessibility', '{}'),
('11111111-1111-1111-1111-111111114003', '11111111-1111-1111-1111-111111111111', 'entertainment', '{}'),
('11111111-1111-1111-1111-111111114005', '11111111-1111-1111-1111-111111111111', 'socials', '{"website_url":"https://korukulture.example.com","instagram_url":"https://www.instagram.com/korukulture?igsh=NGdhemI1d29obXdq","google_maps_url":"https://maps.app.goo.gl/PmtuGGx9YWSMGSGt6"}'),
('11111111-1111-1111-1111-111111114006', '11111111-1111-1111-1111-111111111111', 'taxes', '{"pricing":"included_or_plus_10_service_plus_10_tax"}'),
('11111111-1111-1111-1111-111111114007', '11111111-1111-1111-1111-111111111111', 'parking', '{"motorbikes":"5","cars":"0"}'),
('11111111-1111-1111-1111-111111114008', '11111111-1111-1111-1111-111111111111', 'house_policies', '{}'),
('11111111-1111-1111-1111-111111114009', '11111111-1111-1111-1111-111111111111', 'careers', '{}'),
('11111111-1111-1111-1111-111111114010', '11111111-1111-1111-1111-111111111111', 'b2b', '{}');

insert into business_fact_translations (fact_id, locale, title, content) values
('11111111-1111-1111-1111-111111114001', 'en', 'Takeaway', 'We do not currently offer delivery, but you are welcome to visit us in-store.'),
('11111111-1111-1111-1111-111111114002', 'en', 'Accessibility', 'Accessibility details are not configured in this test profile. Please contact the restaurant directly for specific assistance needs.'),
('11111111-1111-1111-1111-111111114003', 'en', 'Entertainment', 'We do not host live music. We are a quiet cafe environment.'),
('11111111-1111-1111-1111-111111114005', 'en', 'Socials', 'You can find us on Google Maps, Instagram, and our website.'),
('11111111-1111-1111-1111-111111114006', 'en', 'Taxes', 'Prices may be shown as included, or with a 10% service charge and 10% local tax.'),
('11111111-1111-1111-1111-111111114007', 'en', 'Parking', 'We have space for 5 motorbikes in front of the restaurant and no car parking.'),
('11111111-1111-1111-1111-111111114008', 'en', 'House policies', 'House policy details are not configured in this test profile. Please contact the manager directly for dress code, pets, or children-specific questions.'),
('11111111-1111-1111-1111-111111114009', 'en', 'Careers', 'For career inquiries, please email us directly. We cannot process job applications through this chat.'),
('11111111-1111-1111-1111-111111114010', 'en', 'B2B', 'We do not offer franchising or wholesale coffee beans at this time.');

insert into reservations (id, business_id, reference, customer_name, reservation_date, reservation_time, people_count, status) values
('11111111-1111-1111-1111-111111116001', '11111111-1111-1111-1111-111111111111', 'REST-KORU01', 'Maya Chen', '2026-05-20', '10:00', 2, 'confirmed'),
('11111111-1111-1111-1111-111111116002', '11111111-1111-1111-1111-111111111111', 'REST-KORU02', 'Jean Martin', '2026-05-21', '12:00', 4, 'confirmed'),
('11111111-1111-1111-1111-111111116003', '11111111-1111-1111-1111-111111111111', 'REST-KORU03', 'Priya Singh', '2026-05-22', '14:00', 3, 'confirmed');
