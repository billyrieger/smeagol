const REPS: usize = 10;

fn equal_with_offset(
    before: &[smeagol::Position],
    after: &[smeagol::Position],
    x_offset: i64,
    y_offset: i64,
) {
    assert_eq!(before.len(), after.len());
    for (pos0, pos1) in before.iter().zip(after.iter()) {
        assert_eq!(pos0.x + x_offset, pos1.x);
        assert_eq!(pos0.y + y_offset, pos1.y);
    }
}

fn fly(life: &mut smeagol::Life, x_vel: (i64, i64), y_vel: (i64, i64), period: usize) {
    life.set_step_log_2(0);

    let mut before = life.get_alive_cells();
    before.sort();

    for i in 1..=REPS {
        for _ in 0..period {
            life.step();
        }

        let mut after = life.get_alive_cells();
        after.sort();

        let n = (i * period) as i64;
        equal_with_offset(
            &before,
            &after,
            x_vel.0 * n / x_vel.1,
            y_vel.0 * n / y_vel.1,
        );
    }
}

#[test]
fn glider() {
    let mut life = smeagol::Life::from_rle_file("../assets/glider.rle").unwrap();
    fly(&mut life, (1, 4), (1, 4), 4);
}

#[test]
fn sir_robin() {
    let mut life = smeagol::Life::from_rle_file("../assets/sirrobin.rle").unwrap();
    fly(&mut life, (-1, 6), (-2, 6), 6);
}

#[test]
fn weekender() {
    let mut life = smeagol::Life::from_rle_file("../assets/weekender.rle").unwrap();
    fly(&mut life, (0, 7), (-2, 7), 7);
}
