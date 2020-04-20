/*
 * This Source Code Form is subject to the terms of the Mozilla Public License,
 * v. 2.0. If a copy of the MPL was not distributed with this file, You can
 * obtain one at http://mozilla.org/MPL/2.0/.
 */

const REPS: usize = 10;

fn oscillate(life: &mut smeagol::Life, period: usize) {
    life.set_step_log_2(0);

    let before_cells = life.get_alive_cells();
    let before_pop = life.population();

    for i in 1..=REPS {
        for _ in 0..period {
            life.step();
        }
        assert_eq!(life.generation(), (i * period) as u128);

        let after_cells = life.get_alive_cells();
        let after_pop = life.population();

        assert_eq!(before_cells, after_cells);
        assert_eq!(before_pop, after_pop);
    }

    life.set_step_log_2(10);

    let before_cells = life.get_alive_cells();
    let before_pop = life.population();

    for _ in 0..period {
        life.step();
    }

    let after_cells = life.get_alive_cells();
    let after_pop = life.population();

    assert_eq!(before_cells, after_cells);
    assert_eq!(before_pop, after_pop);
}

#[test]
fn figure_eight() {
    let mut life = smeagol::Life::from_rle_file("./assets/figureeight.rle").unwrap();
    oscillate(&mut life, 8);
}

#[test]
fn pentadecathlon() {
    let mut life = smeagol::Life::from_rle_file("./assets/pentadecathlon.rle").unwrap();
    oscillate(&mut life, 15);
}

#[test]
fn pulsar() {
    let mut life = smeagol::Life::from_rle_file("./assets/pulsar.rle").unwrap();
    oscillate(&mut life, 3);
}

#[test]
fn queen_bee_shuttle() {
    let mut life = smeagol::Life::from_rle_file("./assets/queenbeeshuttle.rle").unwrap();
    oscillate(&mut life, 30);
}
