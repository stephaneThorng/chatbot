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
('11111111-1111-1111-1111-111111111916', 'tofu');

insert into menu_item_ingredients (menu_item_id, ingredient_tag_id)
select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('pizza dough', 'cheese', 'tomato') where mi.code = 'pizza'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('lettuce', 'tomato', 'vegetables') where mi.code = 'salad'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('chocolate', 'cake') where mi.code = 'chocolate_cake'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('rice', 'vegetables') where mi.code = 'fried_rice'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('pasta', 'tomato') where mi.code = 'vegetarian_pasta'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('seafood', 'soup') where mi.code = 'seafood_soup'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('beef', 'bun', 'tomato') where mi.code = 'beef_burger'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('chicken', 'curry') where mi.code = 'chicken_satay'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('tofu', 'curry', 'vegetables') where mi.code = 'vegan_curry'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('pasta', 'tomato') where mi.code = 'kids_pasta'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('beef', 'pasta', 'pizza dough', 'chicken', 'seafood', 'cake') where mi.code = 'set_menu'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('rice', 'chicken') where mi.code = 'lunch_special'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('pasta', 'cake') where mi.code = 'kids_menu'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('bun', 'cake') where mi.code = 'breakfast_menu'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('beef', 'pizza dough', 'cake') where mi.code = 'family_menu'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('beef', 'seafood', 'cake') where mi.code = 'tasting_menu'
union all select mi.id, it.id from menu_items mi join ingredient_tags it on it.code in ('cake', 'chocolate') where mi.code = 'dessert_menu';

insert into business_facts (id, business_id, fact_type, metadata)
values (
    '11111111-1111-1111-1111-111111114004',
    '11111111-1111-1111-1111-111111111111',
    'menu_reference',
    '{"website_url":"https://example.com/menu","pdf_url":"https://example.com/menu.pdf"}'
)
on conflict (id) do nothing;

insert into business_fact_translations (fact_id, locale, title, content) values
('11111111-1111-1111-1111-111111114004', 'en', 'Menu reference', 'You can view our full menu online.'),
('11111111-1111-1111-1111-111111114004', 'id', 'Referensi menu', 'Anda dapat melihat menu lengkap kami secara online.')
on conflict (fact_id, locale) do nothing;
