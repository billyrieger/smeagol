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
}

impl State {
    pub fn new_centered(life: smeagol::Life, output_width: u64, output_height: u64) -> Self {
        let alive_cells = life.get_alive_cells();
        if !alive_cells.is_empty() {
            let x_min = alive_cells.iter().map(|(x, _)| x).min().cloned().unwrap();
            let y_min = alive_cells.iter().map(|(_, y)| y).min().cloned().unwrap();
            let x_max = alive_cells.iter().map(|(x, _)| x).max().cloned().unwrap();
            let y_max = alive_cells.iter().map(|(_, y)| y).max().cloned().unwrap();

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
            }
        } else {
            Self {
                life: Arc::new(Mutex::new(life)),
                is_running: Arc::new(AtomicBool::new(false)),
                step: Arc::new(Mutex::new(1)),
                scale: Arc::new(Mutex::new(1)),
                center: Arc::new(Mutex::new((0, 0))),
            }
        }
    }
}

pub trait Action {
    fn register(&self, siv: &mut cursive::Cursive);
}

pub struct KeyCommand<E, F: 'static> {
    event: E,
    on_event: F,
}

impl<E, F> Action for KeyCommand<E, F>
where
    E: Into<cursive::event::Event> + Clone,
    for<'r> F: Fn(&'r mut cursive::Cursive) + Clone,
{
    fn register(&self, siv: &mut cursive::Cursive) {
        siv.add_global_callback(self.event.clone(), self.on_event.clone());
    }
}

impl<E, F> KeyCommand<E, F>
where
    E: Into<cursive::event::Event> + Clone,
    for<'r> F: Fn(&'r mut cursive::Cursive) + Clone,
{
    pub fn new(event: E, on_event: F) -> Self {
        Self { event, on_event }
    }
}

fn toggle_is_running(is_running: &Arc<AtomicBool>) {
    is_running.store(!is_running.load(Ordering::SeqCst), Ordering::SeqCst);
}

fn zoom_out(scale_factor: &Arc<Mutex<u64>>) {
    let mut scale_factor = scale_factor.lock().unwrap();
    if *scale_factor < (1 << 63) {
        *scale_factor <<= 1;
    }
}

fn zoom_in(scale_factor: &Arc<Mutex<u64>>) {
    let mut scale_factor = scale_factor.lock().unwrap();
    if *scale_factor > 1 {
        *scale_factor >>= 1;
    }
}

fn increase_step(step: &Arc<Mutex<u64>>) {
    let mut step = step.lock().unwrap();
    if *step < (1 << 63) {
        *step <<= 1;
    }
}

fn decrease_step(step: &Arc<Mutex<u64>>) {
    let mut step = step.lock().unwrap();
    if *step > 1 {
        *step >>= 1;
    }
}

fn quit(siv: &mut cursive::Cursive) {
    siv.quit()
}

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

pub fn key_commands(state: &State) -> Vec<Box<dyn Action>> {
    vec![
        Box::new(KeyCommand::new(
            ' ',
            enclose!((state) move |_: &mut cursive::Cursive| {
                toggle_is_running(&state.is_running)
            }),
        )),
        Box::new(KeyCommand::new(
            '+',
            enclose!((state) move |_: &mut cursive::Cursive| {
                zoom_in(&state.scale)
            }),
        )),
        Box::new(KeyCommand::new(
            '-',
            enclose!((state) move |_: &mut cursive::Cursive| {
                zoom_out(&state.scale)
            }),
        )),
        Box::new(KeyCommand::new(
            '}',
            enclose!((state) move |_: &mut cursive::Cursive| {
                increase_step(&state.step)
            }),
        )),
        Box::new(KeyCommand::new(
            '{',
            enclose!((state) move |_: &mut cursive::Cursive| {
                decrease_step(&state.step)
            }),
        )),
        Box::new(KeyCommand::new('q', quit)),
    ]
}

pub fn start_smeagol_thread(siv: &mut cursive::Cursive, state: &State) {
    let sink = siv.cb_sink().clone();

    std::thread::spawn(enclose!((state, sink) move || loop {
        std::thread::sleep(std::time::Duration::from_millis(33));
        if state.is_running.load(Ordering::SeqCst) {
            state.life.lock().unwrap().step(*state.step.lock().unwrap());
            // need to send something to trigger a redraw
            sink.send(Box::new(|_: &mut cursive::Cursive| {})).unwrap();
        }
    }));
}
