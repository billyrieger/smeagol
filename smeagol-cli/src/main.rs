use smeagol_cli::{views, State};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

fn main() {
    let mut siv = cursive::Cursive::default();

    let life = Arc::new(Mutex::new(
        smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap(),
    ));
    let is_running = Arc::new(AtomicBool::new(false));
    let step = Arc::new(Mutex::new(1));
    let scale = Arc::new(Mutex::new(1));
    let center = Arc::new(Mutex::new((0, 0)));

    let state = State {
        life: life.clone(),
        is_running: is_running.clone(),
        step: step.clone(),
        scale: scale.clone(),
        center: center.clone(),
    };

    siv.add_fullscreen_layer(views::main_view(&state));

    let sink = siv.cb_sink().clone();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(33));
        if is_running.load(Ordering::SeqCst) {
            life.lock().unwrap().step(*step.lock().unwrap());
            sink.send(Box::new(|_: &mut cursive::Cursive| {})).unwrap();
        }
    });

    for key_command in smeagol_cli::key_commands(&state) {
        key_command.register(&mut siv);
    }

    siv.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let mut siv = cursive::Cursive::dummy();

        let life = Arc::new(Mutex::new(smeagol::Life::new()));
        let is_running = Arc::new(AtomicBool::new(false));
        let step = Arc::new(Mutex::new(1));
        let scale = Arc::new(Mutex::new(1));
        let center = Arc::new(Mutex::new((0, 0)));

        let state = State {
            life: life.clone(),
            is_running: is_running.clone(),
            step: step.clone(),
            scale: scale.clone(),
            center: center.clone(),
        };

        siv.add_fullscreen_layer(views::main_view(&state));

        for key_command in smeagol_cli::key_commands(&state) {
            key_command.register(&mut siv);
        }

        let sink = siv.cb_sink().clone();

        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(33));
            if is_running.load(Ordering::SeqCst) {
                life.lock().unwrap().step(*step.lock().unwrap());
                sink.send(Box::new(|_: &mut cursive::Cursive| {})).unwrap();
            }
        });

        siv.run();
    }
}
