use crate::State;
use itertools::Itertools;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use cursive::view::Scrollable;

lazy_static::lazy_static! {
    static ref KEY_COMMANDS: Vec<KeyCommandGroup> = {
        vec![
            KeyCommandGroup::new(
                "camera",
                vec![
                    KeyCommand::new(
                        vec![Key::Up, Key::Char('k')],
                        Action::PanUp,
                        "pan up"
                    ),
                    KeyCommand::new(
                        vec![Key::Down, Key::Char('j')],
                        Action::PanDown,
                        "pan down"
                    ),
                    KeyCommand::new(
                        vec![Key::Left, Key::Char('h')],
                        Action::PanLeft,
                        "pan left"
                    ),
                    KeyCommand::new(
                        vec![Key::Right, Key::Char('l')],
                        Action::PanRight,
                        "pan right"
                    ),
                    KeyCommand::new(
                        vec![Key::ShiftUp, Key::Char('K')],
                        Action::PanUpSmall,
                        "pan up (small)"
                    ),
                    KeyCommand::new(
                        vec![Key::ShiftDown, Key::Char('J')],
                        Action::PanDownSmall,
                        "pan down (small)"
                    ),
                    KeyCommand::new(
                        vec![Key::ShiftLeft, Key::Char('H')],
                        Action::PanLeftSmall,
                        "pan left (small)"
                    ),
                    KeyCommand::new(
                        vec![Key::ShiftRight, Key::Char('L')],
                        Action::PanRightSmall,
                        "pan right (small)"
                    ),
                    KeyCommand::new(
                        vec![Key::Char('[')],
                        Action::IncreaseScale,
                        "zoom out"
                    ),
                    KeyCommand::new(
                        vec![Key::Char(']')],
                        Action::DecreaseScale,
                        "zoom in"
                    ),
                    KeyCommand::new(
                        vec![Key::Char('f')],
                        Action::ZoomToFit,
                        "zoom to fit (might be slow)"
                    ),
                ]
            ),

            KeyCommandGroup::new(
                "simuate",
                 vec![
                    KeyCommand::new(
                        vec![Key::Enter],
                        Action::ToggleSimulation,
                        "start/stop simulation"
                    ),
                    KeyCommand::new(
                        vec![Key::Char(' ')],
                        Action::StepOneGeneration,
                        "advance one generation"
                    ),
                    KeyCommand::new(
                        vec![Key::Tab],
                        Action::Step,
                        "advance <step> generations"
                    ),
                    KeyCommand::new(
                        vec![Key::Char('-')],
                        Action::DecreaseStep,
                        "decrease step size"
                    ),
                    KeyCommand::new(
                        vec![Key::Char('=')],
                        Action::IncreaseStep,
                        "increase step size"
                    ),
                    KeyCommand::new(
                        vec![Key::Char('9')],
                        Action::DecreaseDelay,
                        "decrease frame delay"
                    ),
                    KeyCommand::new(
                        vec![Key::Char('0')],
                        Action::IncreaseDelay,
                        "increase frame delay"
                    ),
                ]
            ),

            KeyCommandGroup::new(
                "other",
                vec![
                    KeyCommand::new(
                        vec![Key::Char('q')],
                        Action::Quit,
                        "quit"
                    ),
                    KeyCommand::new(
                        vec![Key::Char('?')],
                        Action::ShowHelp,
                        "toggle help"
                    ),
                 ]
             ),
        ]
    };
}

#[derive(Clone, Copy, Debug)]
pub enum Key {
    Char(char),
    Enter,
    Tab,
    Up,
    Down,
    Left,
    Right,
    ShiftUp,
    ShiftDown,
    ShiftLeft,
    ShiftRight,
}

impl Key {
    fn into_event(self) -> cursive::event::Event {
        match self {
            Key::Char(c) => cursive::event::Event::Char(c),
            Key::Enter => cursive::event::Event::Key(cursive::event::Key::Enter),
            Key::Tab => cursive::event::Event::Key(cursive::event::Key::Tab),
            Key::Up => cursive::event::Event::Key(cursive::event::Key::Up),
            Key::Down => cursive::event::Event::Key(cursive::event::Key::Down),
            Key::Left => cursive::event::Event::Key(cursive::event::Key::Left),
            Key::Right => cursive::event::Event::Key(cursive::event::Key::Right),
            Key::ShiftUp => cursive::event::Event::Shift(cursive::event::Key::Up),
            Key::ShiftDown => cursive::event::Event::Shift(cursive::event::Key::Down),
            Key::ShiftLeft => cursive::event::Event::Shift(cursive::event::Key::Left),
            Key::ShiftRight => cursive::event::Event::Shift(cursive::event::Key::Right),
        }
    }

    fn display(self) -> String {
        match self {
            Key::Char(' ') => "<space>".to_owned(),
            Key::Char(c) => format!("{}", c),
            Key::Enter => "<enter>".to_owned(),
            Key::Tab => "<tab>".to_owned(),
            Key::Up => "↑".to_owned(),
            Key::Down => "↓".to_owned(),
            Key::Left => "←".to_owned(),
            Key::Right => "→".to_owned(),
            Key::ShiftUp => "<shift> ↑".to_owned(),
            Key::ShiftDown => "<shift> ↓".to_owned(),
            Key::ShiftLeft => "<shift> ←".to_owned(),
            Key::ShiftRight => "<shift> →".to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Action {
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    PanLeftSmall,
    PanRightSmall,
    PanUpSmall,
    PanDownSmall,
    IncreaseStep,
    DecreaseStep,
    IncreaseScale,
    DecreaseScale,
    ToggleSimulation,
    StepOneGeneration,
    Step,
    IncreaseDelay,
    DecreaseDelay,
    Quit,
    ShowHelp,
    ZoomToFit,
}

#[derive(Clone, Debug)]
pub struct KeyCommand {
    keys: Vec<Key>,
    action: Action,
    description: &'static str,
}

impl KeyCommand {
    fn new(keys: Vec<Key>, action: Action, description: &'static str) -> Self {
        Self {
            keys,
            action,
            description,
        }
    }
}

pub struct KeyCommandGroup {
    name: &'static str,
    key_commands: Vec<KeyCommand>,
}

impl KeyCommandGroup {
    fn new(name: &'static str, key_commands: Vec<KeyCommand>) -> Self {
        Self { name, key_commands }
    }
}

const MOVEMENT_FACTOR: u64 = 4;
const MIN_SCALE: u64 = 1;
const MAX_SCALE: u64 = 1 << 48;
const MAX_STEP: u64 = 1 << 48;

fn pan_down(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.1 += (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn pan_up(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.1 -= (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn pan_left(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.0 -= (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn pan_right(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.0 += (MOVEMENT_FACTOR * *scale.lock().unwrap()) as i64;
}

fn pan_down_small(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.1 += *scale.lock().unwrap() as i64;
}

fn pan_up_small(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.1 -= *scale.lock().unwrap() as i64;
}

fn pan_left_small(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.0 -= *scale.lock().unwrap() as i64;
}

fn pan_right_small(center: &Arc<Mutex<(i64, i64)>>, scale: &Arc<Mutex<u64>>) {
    let mut center = center.lock().unwrap();
    center.0 += *scale.lock().unwrap() as i64;
}

fn step_one_generation(life: &Arc<Mutex<smeagol::Life>>) {
    life.lock().unwrap().step(1);
}

fn step(life: &Arc<Mutex<smeagol::Life>>, step: &Arc<Mutex<u64>>) {
    life.lock().unwrap().step(*step.lock().unwrap());
}

fn toggle_simulation(is_running: &Arc<AtomicBool>) {
    is_running.store(!is_running.load(Ordering::SeqCst), Ordering::SeqCst);
}

fn increase_scale(scale: &Arc<Mutex<u64>>) {
    let mut scale = scale.lock().unwrap();
    if *scale < MAX_SCALE {
        *scale <<= 1;
    }
}

fn decrease_scale(scale: &Arc<Mutex<u64>>) {
    let mut scale = scale.lock().unwrap();
    if *scale > MIN_SCALE {
        *scale >>= 1;
    }
}

fn zoom_to_fit(
    life: &Arc<Mutex<smeagol::Life>>,
    center: &Arc<Mutex<(i64, i64)>>,
    scale: &Arc<Mutex<u64>>,
) {
    let life = life.lock().unwrap();
    if life.population() > 0 {
        let (output_width, output_height) = term_size::dimensions().unwrap();
        let x_min = life.min_alive_x().unwrap();
        let y_min = life.min_alive_y().unwrap();
        let x_max = life.max_alive_x().unwrap();
        let y_max = life.max_alive_y().unwrap();

        let new_center = ((x_min + x_max) / 2, (y_min + y_max) / 2);
        let width = (x_max - x_min + 1) as f64;
        let height = (y_max - y_min + 1) as f64;
        let new_scale = ((width / ((output_width as f64) * 2.))
            .ceil()
            .max((height / (((output_height - 1) as f64) * 4.)).ceil())
            as u64)
            .next_power_of_two();
        *center.lock().unwrap() = new_center;
        *scale.lock().unwrap() = new_scale;
    } else {
        *center.lock().unwrap() = (0, 0);
        *scale.lock().unwrap() = 1;
    }
}

fn increase_step(step: &Arc<Mutex<u64>>) {
    let mut step = step.lock().unwrap();
    if *step < MAX_STEP {
        *step <<= 1;
    }
}

fn decrease_step(step: &Arc<Mutex<u64>>) {
    let mut step = step.lock().unwrap();
    if *step > 1 {
        *step >>= 1;
    }
}

fn increase_delay(delay: &Arc<Mutex<u64>>) {
    let mut delay = delay.lock().unwrap();
    if *delay < (1 << 10) {
        *delay <<= 1;
    }
}

fn decrease_delay(delay: &Arc<Mutex<u64>>) {
    let mut delay = delay.lock().unwrap();
    if *delay > 1 {
        *delay >>= 1;
    }
}

fn quit(siv: &mut cursive::Cursive) {
    siv.quit()
}

fn show_help(siv: &mut cursive::Cursive) {
    let mut stack = siv.find_id::<cursive::views::StackView>("stack").unwrap();
    if stack
        .get(cursive::views::LayerPosition::FromBack(1))
        .is_some()
    {
        stack.pop_layer();
    } else {
        let mut help = cursive::views::LinearLayout::vertical();

        help.add_child(
            cursive::views::TextView::new(format!("smeagol {}", clap::crate_version!())).center(),
        );

        help.add_child(cursive::views::TextView::new(clap::crate_authors!()).center());

        let mut groups = cursive::views::LinearLayout::horizontal();
        for key_command_group in KEY_COMMANDS.iter() {
            let mut group = cursive::views::LinearLayout::vertical();
            group.add_child(cursive::views::TextView::new(" "));
            group.add_child(cursive::views::TextView::new(key_command_group.name).center());
            group.add_child(cursive::views::TextView::new(" "));
            let mut help_list = cursive::views::ListView::new();
            for key_command in &key_command_group.key_commands {
                let label = key_command
                    .keys
                    .iter()
                    .map(|key| key.display())
                    .intersperse(", ".to_owned())
                    .collect::<String>();
                help_list.add_child(
                    &label,
                    cursive::views::TextView::new(key_command.description),
                );
            }
            group.add_child(help_list);
            groups.add_child(cursive::views::PaddedView::new(((1, 1), (0, 0)), group));
        }
        help.add_child(groups);
        stack.add_layer(cursive::views::IdView::new(
            "help",
            cursive::views::PaddedView::new(((2, 2), (1, 1)), help.scrollable().scroll_x(true).scroll_y(true)),
        ));
    }
}

pub fn setup_key_commands(siv: &mut cursive::Cursive, state: &State) {
    for key_command_group in KEY_COMMANDS.iter() {
        for key_command in key_command_group.key_commands.iter() {
            for &key in &key_command.keys {
                match key_command.action {
                    Action::PanDown => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_down(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::PanUp => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_up(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::PanLeft => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_left(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::PanRight => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_right(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::PanDownSmall => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_down_small(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::PanUpSmall => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_up_small(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::PanLeftSmall => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_left_small(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::PanRightSmall => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                pan_right_small(&state.center, &state.scale)
                            }),
                        );
                    }
                    Action::IncreaseScale => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                increase_scale(&state.scale)
                            }),
                        );
                    }
                    Action::DecreaseScale => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                decrease_scale(&state.scale)
                            }),
                        );
                    }
                    Action::ZoomToFit => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                zoom_to_fit(&state.life, &state.center, &state.scale)
                            }),
                        );
                    }
                    Action::IncreaseStep => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                increase_step(&state.step)
                            }),
                        );
                    }
                    Action::DecreaseStep => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                decrease_step(&state.step)
                            }),
                        );
                    }
                    Action::StepOneGeneration => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                step_one_generation(&state.life)
                            }),
                        );
                    }
                    Action::Step => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                step(&state.life, &state.step)
                            }),
                        );
                    }
                    Action::ToggleSimulation => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                toggle_simulation(&state.is_running)
                            }),
                        );
                    }
                    Action::IncreaseDelay => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                increase_delay(&state.delay_millis)
                            }),
                        );
                    }
                    Action::DecreaseDelay => {
                        siv.add_global_callback(
                            key.into_event(),
                            enclose!((state) move |_: &mut cursive::Cursive| {
                                decrease_delay(&state.delay_millis)
                            }),
                        );
                    }
                    Action::Quit => {
                        siv.add_global_callback(key.into_event(), quit);
                    }
                    Action::ShowHelp => {
                        siv.add_global_callback(key.into_event(), show_help);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_scale() {
        let scale = Arc::new(Mutex::new(8));

        increase_scale(&scale);
        assert_eq!(*scale.lock().unwrap(), 16);

        decrease_scale(&scale);
        assert_eq!(*scale.lock().unwrap(), 8);

        let min_scale = Arc::new(Mutex::new(MIN_SCALE));
        decrease_scale(&min_scale);
        assert_eq!(*min_scale.lock().unwrap(), MIN_SCALE);

        let max_scale = Arc::new(Mutex::new(MAX_SCALE));
        increase_scale(&max_scale);
        assert_eq!(*max_scale.lock().unwrap(), MAX_SCALE);
    }

    #[test]
    fn pan() {
        let center = Arc::new(Mutex::new((0, 0)));
        let scale = Arc::new(Mutex::new(4));

        pan_down(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (0, 4 * MOVEMENT_FACTOR as i64));

        pan_up(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (0, 0));

        pan_right(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (4 * MOVEMENT_FACTOR as i64, 0));

        pan_left(&center, &scale);
        assert_eq!(*center.lock().unwrap(), (0, 0));
    }

    #[test]
    fn dummy_setup_key_commands() {
        let mut siv = cursive::Cursive::dummy();
        let life = smeagol::Life::new();
        let state = State::new_centered(life, 20, 20);
        setup_key_commands(&mut siv, &state);
    }
}
