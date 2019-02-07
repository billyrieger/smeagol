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
    ]
}
