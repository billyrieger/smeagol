fn make_glider() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
bob$2bo$3o!
",
    )
    .unwrap()
}

fn make_sir_robin() -> smeagol::Life {
    smeagol::Life::from_rle_pattern("
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

fn equal_with_offset(before: &[(i64, i64)], after: &[(i64, i64)], x_offset: i64, y_offset: i64) {
    assert_eq!(before.len(), after.len());
    for (&(x0, y0), &(x1, y1)) in before.iter().zip(after.iter()) {
        assert_eq!(x0 + x_offset, x1);
        assert_eq!(y0 + y_offset, y1);
    }
}

fn helper(mut life: smeagol::Life, x_speed: (i64, i64), y_speed: (i64, i64), period: u64) {
    let mut before = life.get_alive_cells();
    before.sort();

    life.step(period);

    let mut after = life.get_alive_cells();
    after.sort();

    equal_with_offset(
        &before,
        &after,
        x_speed.0 * (period as i64) / x_speed.1,
        y_speed.0 * ( period as i64) / y_speed.1,
    );
}

#[test]
fn glider() {
    helper(make_glider(), (1, 4), (1, 4), 4);
}

#[test]
fn sir_robin() {
    helper(make_sir_robin(), (-1, 6), (-2, 6), 6);
}