fn make_figure_eight() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
2o4b$2obo2b$4bob$bo4b$2bob2o$4b2o!
",
    )
    .unwrap()
}

fn make_pentadecathlon() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
2bo4bo2b$2ob4ob2o$2bo4bo!
",
    )
    .unwrap()
}

fn make_pulsar() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
2b3o3b3o2b2$o4bobo4bo$o4bobo4bo$o4bobo4bo$2b3o3b3o2b2$2b3o3b3o2b$o4bob
o4bo$o4bobo4bo$o4bobo4bo2$2b3o3b3o!
",
    )
    .unwrap()
}

fn make_queen_bee_shuttle() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
9bo12b$7bobo12b$6bobo13b$2o3bo2bo11b2o$2o4bobo11b2o$7bobo12b$9bo!
",
    )
    .unwrap()
}

fn helper(mut life: smeagol::Life, period: u64) {
    let mut before = life.get_alive_cells();
    let before_population = life.population();
    let before_generation = life.generation();
    before.sort();

    life.step(period);

    let mut after = life.get_alive_cells();
    let after_population = life.population();
    let after_generation = life.generation();
    after.sort();

    assert_eq!(before, after);
    assert_eq!(before_population, after_population);
    assert_eq!(after_generation - before_generation, period as u128);

    for _ in 0..period {
        life.step(1);
    }

    let mut after_again = life.get_alive_cells();
    let after_again_population = life.population();
    let after_again_generation = life.generation();
    after_again.sort();

    assert_eq!(before, after_again);
    assert_eq!(before_population, after_again_population);
    assert_eq!(after_again_generation - after_generation, period as u128);
}

#[test]
fn figure_eight() {
    helper(make_figure_eight(), 8);
}

#[test]
fn pentadecathlon() {
    helper(make_pentadecathlon(), 15);
}

#[test]
fn pulsar() {
    helper(make_pulsar(), 3);
}

#[test]
fn queen_bee_shuttle() {
    helper(make_queen_bee_shuttle(), 30);
}
