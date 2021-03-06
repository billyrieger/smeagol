/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

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

    let mut before_cells = life.get_alive_cells();
    before_cells.sort();
    let before_pop = life.population();

    for i in 1..=REPS {
        for _ in 0..period {
            life.step();
        }
        assert_eq!(life.generation(), (i * period) as u128);

        let mut after_cells = life.get_alive_cells();
        after_cells.sort();
        let after_pop = life.population();

        let n = (i * period) as i64;
        equal_with_offset(
            &before_cells,
            &after_cells,
            x_vel.0 * n / x_vel.1,
            y_vel.0 * n / y_vel.1,
        );
        assert_eq!(before_pop, after_pop);
    }

    life.set_step_log_2(10);

    let mut before_cells = life.get_alive_cells();
    before_cells.sort();
    let before_pop = life.population();

    for _ in 0..period {
        life.step();
    }

    let mut after_cells = life.get_alive_cells();
    after_cells.sort();
    let after_pop = life.population();

    let n = (1024 * period) as i64;
    equal_with_offset(
        &before_cells,
        &after_cells,
        x_vel.0 * n / x_vel.1,
        y_vel.0 * n / y_vel.1,
    );
    assert_eq!(before_pop, after_pop);
}

#[test]
fn glider() {
    let mut life = smeagol::Life::from_rle_file("./assets/glider.rle").unwrap();
    fly(&mut life, (1, 4), (1, 4), 4);
}

#[test]
fn sir_robin() {
    let mut life = smeagol::Life::from_rle_file("./assets/sirrobin.rle").unwrap();
    fly(&mut life, (-1, 6), (-2, 6), 6);
}

#[test]
fn spaghetti_monster() {
    let mut life = smeagol::Life::from_rle_file("./assets/spaghettimonster.rle").unwrap();
    fly(&mut life, (0, 7), (-3, 7), 7);
}

#[test]
fn weekender() {
    let mut life = smeagol::Life::from_rle_file("./assets/weekender.rle").unwrap();
    fly(&mut life, (0, 7), (-2, 7), 7);
}
