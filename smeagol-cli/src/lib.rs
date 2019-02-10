macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub mod key;
pub mod views;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

#[derive(Clone)]
pub struct State {
    pub life: Arc<Mutex<smeagol::Life>>,
    pub is_running: Arc<AtomicBool>,
    pub step: Arc<Mutex<u64>>,
    pub scale: Arc<Mutex<u64>>,
    pub center: Arc<Mutex<(i64, i64)>>,
    pub delay_millis: Arc<Mutex<u64>>,
}

impl State {
    pub fn new_centered(life: smeagol::Life, output_width: u64, output_height: u64) -> Self {
        if life.population() > 0 {
            let x_min = life.min_alive_x().unwrap();
            let y_min = life.min_alive_y().unwrap();
            let x_max = life.max_alive_x().unwrap();
            let y_max = life.max_alive_y().unwrap();

            let center = ((x_min + x_max) / 2, (y_min + y_max) / 2);
            let width = (x_max - x_min + 1) as f64;
            let height = (y_max - y_min + 1) as f64;
            let scale = ((width / ((output_width as f64) * 2.))
                .ceil()
                .max((height / ((output_height as f64) * 4.)).ceil())
                as u64)
                .next_power_of_two();

            Self {
                life: Arc::new(Mutex::new(life)),
                is_running: Arc::new(AtomicBool::new(false)),
                step: Arc::new(Mutex::new(1)),
                scale: Arc::new(Mutex::new(scale)),
                center: Arc::new(Mutex::new(center)),
                delay_millis: Arc::new(Mutex::new(32)),
            }
        } else {
            Self {
                life: Arc::new(Mutex::new(life)),
                is_running: Arc::new(AtomicBool::new(false)),
                step: Arc::new(Mutex::new(1)),
                scale: Arc::new(Mutex::new(1)),
                center: Arc::new(Mutex::new((0, 0))),
                delay_millis: Arc::new(Mutex::new(32)),
            }
        }
    }
}

pub fn start_smeagol_thread(siv: &mut cursive::Cursive, state: &State) {
    let sink = siv.cb_sink().clone();

    std::thread::spawn(enclose!((state, sink) move || loop {
        let delay = state.delay_millis.lock().unwrap().clone();
        std::thread::sleep(std::time::Duration::from_millis(delay));
        if state.is_running.load(Ordering::SeqCst) {
            state.life.lock().unwrap().step(*state.step.lock().unwrap());
            // need to send something to trigger a redraw
            sink.send(Box::new(|_: &mut cursive::Cursive| {})).unwrap();
        }
    }));
}
