create table ingredient_tags (
    id uuid primary key,
    code text unique not null
);

create table menu_item_ingredients (
    menu_item_id uuid not null references menu_items(id),
    ingredient_tag_id uuid not null references ingredient_tags(id),
    primary key (menu_item_id, ingredient_tag_id)
);

insert into ingredient_tags (id, code) values
('11111111-1111-1111-1111-111111111901', 'beef'),
('11111111-1111-1111-1111-111111111902', 'bun'),
('11111111-1111-1111-1111-111111111903', 'lettuce'),
('11111111-1111-1111-1111-111111111904', 'tomato'),
('11111111-1111-1111-1111-111111111905', 'chocolate'),
('11111111-1111-1111-1111-111111111906', 'rice'),
('11111111-1111-1111-1111-111111111907', 'pasta'),
('11111111-1111-1111-1111-111111111908', 'seafood'),
('11111111-1111-1111-1111-111111111909', 'chicken'),
('11111111-1111-1111-1111-111111111910', 'curry'),
('11111111-1111-1111-1111-111111111911', 'cheese'),
('11111111-1111-1111-1111-111111111912', 'cake'),
('11111111-1111-1111-1111-111111111913', 'vegetables'),
('11111111-1111-1111-1111-111111111914', 'soup'),
('11111111-1111-1111-1111-111111111915', 'pizza dough'),
('11111111-1111-1111-1111-111111111916', 'tofu'),
('11111111-1111-1111-1111-111111111917', 'green papaya'),
('11111111-1111-1111-1111-111111111918', 'carrots'),
('11111111-1111-1111-1111-111111111919', 'cherry tomatoes'),
('11111111-1111-1111-1111-111111111920', 'lime dressing'),
('11111111-1111-1111-1111-111111111921', 'spring roll wrapper'),
('11111111-1111-1111-1111-111111111922', 'sweet and sour sauce'),
('11111111-1111-1111-1111-111111111923', 'peanut sauce'),
('11111111-1111-1111-1111-111111111924', 'fried egg'),
('11111111-1111-1111-1111-111111111925', 'prawn crackers'),
('11111111-1111-1111-1111-111111111926', 'coconut milk'),
('11111111-1111-1111-1111-111111111927', 'wagyu beef'),
('11111111-1111-1111-1111-111111111928', 'aged cheddar'),
('11111111-1111-1111-1111-111111111929', 'spicy mayo'),
('11111111-1111-1111-1111-111111111930', 'french fries'),
('11111111-1111-1111-1111-111111111931', 'mahi-mahi'),
('11111111-1111-1111-1111-111111111932', 'sambal matah'),
('11111111-1111-1111-1111-111111111933', 'mango'),
('11111111-1111-1111-1111-111111111934', 'sticky rice'),
('11111111-1111-1111-1111-111111111935', 'coconut cream'),
('11111111-1111-1111-1111-111111111936', 'vanilla ice cream'),
('11111111-1111-1111-1111-111111111937', 'tea'),
('11111111-1111-1111-1111-111111111938', 'ginger'),
('11111111-1111-1111-1111-111111111939', 'lemon'),
('11111111-1111-1111-1111-111111111940', 'beer');

insert into menu_item_ingredients (menu_item_id, ingredient_tag_id)
select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('chicken', 'vegetables', 'spring roll wrapper', 'sweet and sour sauce') where mi.code = 'chicken_spring_rolls'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('green papaya', 'carrots', 'cherry tomatoes', 'lime dressing') where mi.code = 'green_papaya_salad'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('chicken', 'peanut sauce') where mi.code = 'chicken_satay_skewers'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('rice', 'fried egg', 'chicken', 'prawn crackers') where mi.code = 'nasi_goreng_special'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('coconut milk', 'vegetables', 'tofu', 'rice', 'curry') where mi.code = 'yellow_tofu_curry'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('wagyu beef', 'beef', 'aged cheddar', 'bun', 'spicy mayo', 'french fries') where mi.code = 'bali_wagyu_burger'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('mahi-mahi', 'sambal matah', 'rice') where mi.code = 'grilled_mahi_mahi'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('mango', 'sticky rice', 'coconut cream') where mi.code = 'mango_sticky_rice'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('chocolate', 'cake', 'vanilla ice cream') where mi.code = 'chocolate_lava_cake'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('tea', 'ginger', 'lemon') where mi.code = 'homemade_kombucha'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('beer', 'lemon') where mi.code = 'bintang_radler';

insert into business_facts (id, business_id, fact_type, metadata)
values (
    '11111111-1111-1111-1111-111111114004',
    '11111111-1111-1111-1111-111111111111',
    'menu_reference',
    '{"website_url":"https://korukulture.example.com/menu","pdf_url":"https://korukulture.example.com/menu.pdf"}'
)
on conflict (id) do nothing;

insert into business_fact_translations (fact_id, locale, title, content) values
('11111111-1111-1111-1111-111111114004', 'en', 'Menu reference', 'You can view our full menu online.'),
('11111111-1111-1111-1111-111111114004', 'id', 'Referensi menu', 'Anda dapat melihat menu lengkap kami secara online.')
on conflict (fact_id, locale) do nothing;
