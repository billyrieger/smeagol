fn figure_eight() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
2o4b$2obo2b$4bob$bo4b$2bob2o$4b2o!
",
    )
    .unwrap()
}

fn pentadecathlon() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
2bo4bo2b$2ob4ob2o$2bo4bo!
",
    )
    .unwrap()
}

fn pulsar() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
2b3o3b3o2b2$o4bobo4bo$o4bobo4bo$o4bobo4bo$2b3o3b3o2b2$2b3o3b3o2b$o4bob
o4bo$o4bobo4bo$o4bobo4bo2$2b3o3b3o!
",
    )
    .unwrap()
}

fn queen_bee_shuttle() -> smeagol::Life {
    smeagol::Life::from_rle_pattern(
        "
9bo12b$7bobo12b$6bobo13b$2o3bo2bo11b2o$2o4bobo11b2o$7bobo12b$9bo!
",
    )
    .unwrap()
}

fn helper(mut life: smeagol::Life, period: u64) {
    let mut before = life.get_alive_cells();
    before.sort();

    life.step(period);

    let mut after = life.get_alive_cells();
    after.sort();

    assert_eq!(before, after);

    for _ in 0..period {
        life.step(1);
    }

    let mut after_again = life.get_alive_cells();
    after_again.sort();

    assert_eq!(before, after_again);
}

#[test]
fn figure_eight_period_8() {
    helper(figure_eight(), 8);
}

#[test]
fn pentadecathlon_period_15() {
    helper(pentadecathlon(), 15);
}

#[test]
fn pulsar_period_3() {
    helper(pulsar(), 3);
}

#[test]
fn queen_bee_shuttle_period_30() {
    helper(queen_bee_shuttle(), 30);
}
