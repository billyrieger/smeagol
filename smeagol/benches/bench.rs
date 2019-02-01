extern crate criterion;

fn glider() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        b"
bob$2bo$3o!
",
    )
    .unwrap()
}

fn gosper_glider_gun() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        b"
24bo11b$22bobo11b$12b2o6b2o12b2o$11bo3bo4b2o12b2o$2o8bo5bo3b2o14b$2o8b
o3bob2o4bobo11b$10bo5bo7bo11b$11bo3bo20b$12b2o!
",
    )
    .unwrap()
}

fn sir_robin() -> smeagol::Life {
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
23b2o3b3o$24b2ob2o$25b2o$25bo2$24b2o$26bo!
",
    )
    .unwrap()
}

fn create_glider(c: &mut criterion::Criterion) {
    c.bench_function("create glider", |b| b.iter(|| glider()));
}

fn create_gosper_glider_gun(c: &mut criterion::Criterion) {
    c.bench_function("create gosper glider gun", |b| {
        b.iter(|| gosper_glider_gun())
    });
}

fn create_sir_robin(c: &mut criterion::Criterion) {
    c.bench_function("create sir robin", |b| b.iter(|| sir_robin()));
}

fn step_glider(c: &mut criterion::Criterion) {
    c.bench_function("step glider", move |b| {
        b.iter(|| {
            let mut glider = glider();
            for _ in 0..32 {
                glider.step_pow_2(5);
            }
        })
    });
}

fn step_gosper_glider_gun(c: &mut criterion::Criterion) {
    c.bench_function("step gosper glider gun", move |b| {
        b.iter(|| {
            let mut gun = gosper_glider_gun();
            for _ in 0..1 {
                gun.step_pow_2(10);
            }
        })
    });
}

fn step_sir_robin(c: &mut criterion::Criterion) {
    c.bench_function("step sir robin", move |b| {
        b.iter(|| {
            let mut sir_robin = sir_robin();
            for _ in 0..1 {
                sir_robin.step_pow_2(10);
            }
        })
    });
}

criterion::criterion_group!(
    name = benches;
    config = criterion::Criterion::default();
    targets =
        create_glider,
        create_gosper_glider_gun,
        create_sir_robin,
        step_glider,
        step_gosper_glider_gun,
        step_sir_robin,
);
criterion::criterion_main!(benches);
