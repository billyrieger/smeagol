#[macro_use]
extern crate criterion;

fn create_glider() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(b"bob$2bo$3o!").unwrap()
}

fn create_gosper_glider_gun() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        b"
24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8b
o3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!",
    )
    .unwrap()
}

fn create_sir_robin() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        b"
4b2o$4bo2bo$4bo3bo$6b3o$2b2o6b4o$2bob2o4b4o$bo4bo6b3o$2b4o4b2o3bo$o9b
2o$bo3bo$6b3o2b2o2bo$2b2o7bo4bo$13bob2o$10b2o6bo$11b2ob3obo$10b2o3bo2b
o$10bobo2b2o$10bo2bobobo$10b3o6bo$11bobobo3bo$14b2obobo$11bo6b3o2$11bo
9bo$11bo3bo6bo$12bo5b5o$12b3o$16b2o$13b3o2bo$11bob3obo$10bo3bo2bo$11bo
4b2ob3o$13b4obo4b2o$13bob4o4b2o$19bo$20bo2b2o$20b2o$21b5o$25b2o$19b3o
6bo$20bobo3bobo$19bo3bo3bo$19bo3b2o$18bo6bob3o$19b2o3bo3b2o$20b4o2bo2b
o$22b2o3bo$21bo$21b2obo$20bo$19b5o$19bo4bo$18b3ob3o$18bob5o$18bo$20bo$
16bo4b4o$20b4ob2o$17b3o4bo$24bobo$28bo$24bo2b2o$25b3o$22b2o$21b3o5bo$
24b2o2bobo$21bo2b3obobo$22b2obo2bo$24bobo2b2o$26b2o$22b3o4bo$22b3o4bo$
23b2o3b3o$24b2ob2o$25b2o$25bo2$24b2o$26bo!",
    )
    .unwrap()
}

fn create_spaghetti_monster() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        b"
8b3o5b3o$8bobo5bobo$8bobo5bobo$6bob2o3bo3b2obo$6b2o4bobo4b2o$10b2obob
2o$9bo7bo$9bobo3bobo$5b5o7b5o$4bo2bo11bo2bo$5bob3o7b3obo$7bob2o5b2obo$
6b2obobo3bobob2o$6b3obo5bob3o2$10b2o3b2o$12bobo$9bo7bo$9b2o5b2o$6b2o
11b2o$4bob2o11b2obo$4b2o2b2o7b2o2b2o$4bo2bo2bo5bo2bo2bo$5bo4bo5bo4bo$
5bo2bo2bo3bo2bo2bo$2bo5bo9bo5bo$3bobo15bobo$7bo11bo$3bo3bobo7bobo3bo$
3bo2bo3bo5bo3bo2bo$4b2o2b2o7b2o2b2o$8bo9bo2$8b5ob5o$bo6b2ob2ob2ob2o6bo
$3o7bo5bo7b3o$o2b2o5bo5bo5b2o2bo$2bo3b5o5b5o3bo$7bob2o5b2obo$bo3bo15bo
3bo$bob2o2bo11bo2b2obo$bob4o13b4obo$4bo17bo2$2bo21bo$bobo19bobo$o25bo$
o3bo17bo3bo$5bo15bo$2o23b2o$2bo3bo2bo7bo2bo3bo$2bo3bobobo5bobobo3bo$2b
o5bob2o3b2obo5bo$2bo3b2obo7bob2o3bo$6b2o11b2o$4bo17bo$3bo19bo$3bo4bo9b
o4bo$2b2o3b2o9b2o3b2o$2b2o3bobo7bobo3b2o$2b2o3b2o3b3o3b2o3b2o$2b3o2b3o
bo3bob3o2b3o$6bob2obo3bob2obo$2b2o3b2obo5bob2o3b2o$3bob2o3bobobobo3b2o
bo$11bobobo$8bo9bo$8b3o5b3o$10b2obob2o$10b7o$8b3o5b3o$7b2obobobobob2o$
6bo3bo5bo3bo$11b2ob2o$5bo2bobobobobobo2bo$6b4o7b4o$9bo7bo$9bo7bo$6b2ob
o2bobo2bob2o2$9b2o5b2o3$9bo7bo$9b3o3b3o$8bo2bo3bo2bo$9bo7bo$8bo2bo3bo
2bo$11b2ob2o$12bobo$10bobobobo$9bo3bo3bo$9bo7bo$12bobo$7b2obo5bob2o$7b
2o2bo3bo2b2o$7bo11bo$8bo9bo$6bobo9bobo$5b4o9b4o$5b2obobo5bobob2o$4bo2b
o11bo2bo$9bobo3bobo$8b2obo3bob2o$4bo2bo3b2ob2o3bo2bo$9bo2bobo2bo$6bo2b
ob2ob2obo2bo$7bobobobobobobo$8b2o2bobo2b2o$9bobo3bobo$10b2o3b2o$7b2o9b
2o$7b3o7b3o$7bobo7bobo$5b2o2bo7bo2b2o$5b2o13b2o$11bo3bo$6bo4bo3bo4bo$
6b2o3bo3bo3b2o$7bo2bo5bo2bo$7b3o7b3o$6bobo9bobo$6b2o11b2o$6bobo4bo4bob
o$6b2o4b3o4b2o$6b2o3bo3bo3b2o$5b3o4b3o4b3o$3b2o17b2o$2bo5b2o2bobo2b2o
5bo2$2bo2bob3ob2ob2ob3obo2bo$8b3o5b3o$10b3ob3o$5bo4b2obob2o4bo$11bo3bo
2$11b2ob2o!
        ",
    )
    .unwrap()
}

fn bench_create_glider(c: &mut criterion::Criterion) {
    c.bench_function("create glider", |b| b.iter(|| create_glider()));
}

fn bench_create_queen_bee_shuttle(c: &mut criterion::Criterion) {
    c.bench_function("create gosper glider gun", |b| {
        b.iter(|| create_gosper_glider_gun())
    });
}

fn bench_create_sir_robin(c: &mut criterion::Criterion) {
    c.bench_function("create sir robin", |b| b.iter(|| create_sir_robin()));
}

fn bench_create_spaghetti_monster(c: &mut criterion::Criterion) {
    c.bench_function("create spaghetti monster", |b| {
        b.iter(|| create_spaghetti_monster())
    });
}

fn bench_step_glider_1(c: &mut criterion::Criterion) {
    c.bench_function("step glider 1", |b| {
        b.iter(|| {
            let mut life = create_glider();
            life.step();
        })
    });
}

fn bench_step_gosper_glider_gun_1(c: &mut criterion::Criterion) {
    c.bench_function("step gosper glider gun 1", |b| {
        b.iter(|| {
            let mut life = create_gosper_glider_gun();
            life.step();
        })
    });
}

fn bench_step_sir_robin_1(c: &mut criterion::Criterion) {
    c.bench_function("step sir robin 1", |b| {
        b.iter(|| {
            let mut life = create_sir_robin();
            life.step();
        })
    });
}

fn bench_step_spaghetti_monster_1(c: &mut criterion::Criterion) {
    c.bench_function("step spaghetti monster 1", |b| {
        b.iter(|| {
            let mut life = create_spaghetti_monster();
            life.step();
        })
    });
}

fn bench_step_glider_1024(c: &mut criterion::Criterion) {
    c.bench_function("step glider 1024", |b| {
        b.iter(|| {
            let mut life = create_glider();
            life.set_step_log_2(10);
            life.step();
        })
    });
}

fn bench_step_gosper_glider_gun_1024(c: &mut criterion::Criterion) {
    c.bench_function("step gosper glider gun 1024", |b| {
        b.iter(|| {
            let mut life = create_gosper_glider_gun();
            life.set_step_log_2(10);
            life.step();
        })
    });
}

fn bench_step_sir_robin_1024(c: &mut criterion::Criterion) {
    c.bench_function("step sir robin 1024", |b| {
        b.iter(|| {
            let mut life = create_sir_robin();
            life.set_step_log_2(10);
            life.step();
        })
    });
}

fn bench_step_spaghetti_monster_1024(c: &mut criterion::Criterion) {
    c.bench_function("step spaghetti monster 1024", |b| {
        b.iter(|| {
            let mut life = create_spaghetti_monster();
            life.set_step_log_2(10);
            life.step();
        })
    });
}

criterion_group!(
    benches,
    bench_create_glider,
    bench_create_queen_bee_shuttle,
    bench_create_sir_robin,
    bench_create_spaghetti_monster,
    bench_step_glider_1,
    bench_step_gosper_glider_gun_1,
    bench_step_sir_robin_1,
    bench_step_spaghetti_monster_1,
    bench_step_glider_1024,
    bench_step_gosper_glider_gun_1024,
    bench_step_sir_robin_1024,
    bench_step_spaghetti_monster_1024,
);
criterion_main!(benches);
