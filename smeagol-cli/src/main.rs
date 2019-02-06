use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

struct GenerationView {
    life: Arc<Mutex<smeagol::Life>>,
}

impl GenerationView {
    fn new(life: Arc<Mutex<smeagol::Life>>) -> Self {
        Self { life }
    }
}

impl cursive::view::View for GenerationView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print(
            (0, 0),
            &format!("gen: {}", self.life.lock().unwrap().generation()),
        );
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        (
            self.life.lock().unwrap().generation().to_string().len() + 6,
            1,
        )
            .into()
    }
}

struct PopulationView {
    life: Arc<Mutex<smeagol::Life>>,
}

impl PopulationView {
    fn new(life: Arc<Mutex<smeagol::Life>>) -> Self {
        Self { life }
    }
}

impl cursive::view::View for PopulationView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print(
            (0, 0),
            &format!("pop: {}", self.life.lock().unwrap().population()),
        );
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        (
            self.life.lock().unwrap().population().to_string().len() + 6,
            1,
        )
            .into()
    }
}

struct JumpView {
    jump_factor: Arc<Mutex<u8>>,
}

impl JumpView {
    fn new(jump_factor: Arc<Mutex<u8>>) -> Self {
        Self { jump_factor }
    }
}

impl cursive::view::View for JumpView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print(
            (0, 0),
            &format!(
                "jump: {}",
                (1 << *self.jump_factor.lock().unwrap()).to_string()
            ),
        );
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        (
            (1 << *self.jump_factor.lock().unwrap()).to_string().len() + 7,
            1,
        )
            .into()
    }
}

struct LifeView {
    life: Arc<Mutex<smeagol::Life>>,
    center: (i64, i64),
}

impl LifeView {
    fn new(life: Arc<Mutex<smeagol::Life>>) -> Self {
        Self {
            life,
            center: (0, 0),
        }
    }
}

impl cursive::view::View for LifeView {
    fn draw(&self, printer: &cursive::Printer) {
        let width = printer.output_size.x;
        let height = printer.output_size.y;
        let front_color = cursive::theme::Color::Light(cursive::theme::BaseColor::White);
        let back_color = cursive::theme::Color::Dark(cursive::theme::BaseColor::Black);
        let mut life = self.life.lock().unwrap();
        for x in 0..width {
            for y in 0..height {
                printer.with_color(
                    cursive::theme::ColorStyle::new(front_color, back_color),
                    |printer| {
                        printer.print((x as u32, y as u32), {
                            let x_offset = 2 * (x as i64 + self.center.0 - (width / 2) as i64);
                            let y_offset = 4 * (y as i64 + self.center.1 - (height / 2) as i64);
                            let a = if life.contains_alive_cells((x_offset, y_offset), (x_offset, y_offset)) {
                                1
                            } else {
                                0
                            };
                            let b = if life.contains_alive_cells((x_offset + 1, y_offset), (x_offset + 1, y_offset)) {
                                1
                            } else {
                                0
                            };
                            let c = if life.contains_alive_cells((x_offset, y_offset + 1), (x_offset, y_offset + 1)) {
                                1
                            } else {
                                0
                            };
                            let d = if life.contains_alive_cells((x_offset + 1, y_offset + 1), (x_offset + 1, y_offset + 1)) {
                                1
                            } else {
                                0
                            };
                            let e = if life.contains_alive_cells((x_offset, y_offset + 2), (x_offset, y_offset + 2)) {
                                1
                            } else {
                                0
                            };
                            let f = if life.contains_alive_cells((x_offset + 1, y_offset + 2), (x_offset + 1, y_offset + 2)) {
                                1
                            } else {
                                0
                            };
                            let g = if life.contains_alive_cells((x_offset, y_offset + 3), (x_offset, y_offset + 3)) {
                                1
                            } else {
                                0
                            };
                            let h = if life.contains_alive_cells((x_offset + 1, y_offset + 3), (x_offset + 1, y_offset + 3)) {
                                1
                            } else {
                                0
                            };
                            &braille::BRAILLE[a][b][c][d][e][f][g][h].to_string()
                        })
                    },
                );
            }
        }
    }

    fn on_event(&mut self, event: cursive::event::Event) -> cursive::event::EventResult {
        match event {
            cursive::event::Event::Key(cursive::event::Key::Left) => {
                self.center.0 -= 1;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Key(cursive::event::Key::Right) => {
                self.center.0 += 1;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Key(cursive::event::Key::Up) => {
                self.center.1 -= 1;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Key(cursive::event::Key::Down) => {
                self.center.1 += 1;
                cursive::event::EventResult::Consumed(None)
            }
            _ => cursive::event::EventResult::Ignored,
        }
    }
}

// Shamelessly stolen from webplatform's TodoMVC example.
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

fn main() {
    let mut siv = cursive::Cursive::default();

    let life = Arc::new(Mutex::new(
        smeagol::Life::from_rle_file("/home/billy/Downloads/all/breeder1.rle").unwrap(),
    ));
    let is_running = Arc::new(AtomicBool::new(false));
    let jump_factor = Arc::new(Mutex::new(0u8));

    siv.add_fullscreen_layer(
        cursive::views::LinearLayout::vertical()
            .child(cursive::view::Boxable::full_screen(LifeView::new(
                life.clone(),
            )))
            .child(
                cursive::views::LinearLayout::horizontal()
                    .child(GenerationView::new(life.clone()))
                    .child(PopulationView::new(life.clone()))
                    .child(JumpView::new(jump_factor.clone())),
            ),
    );
    siv.add_global_callback(
        ' ',
        enclose!((is_running) move |_| {
            is_running.store(!is_running.load(Ordering::SeqCst), Ordering::SeqCst)
        }),
    );
    siv.add_global_callback(
        '+',
        enclose!((jump_factor) move |_| {
            let mut j = jump_factor.lock().unwrap();
            *j += 1;
        }),
    );
    siv.add_global_callback(
        '-',
        enclose!((jump_factor) move |_| {
            let mut j = jump_factor.lock().unwrap();
            if *j > 0 {
                *j -= 1;
            }
        }),
    );
    siv.add_global_callback(
        cursive::event::Key::Tab,
        enclose!((life) move |_| life.lock().unwrap().step(32)),
    );
    siv.set_fps(30);
    let sink = siv.cb_sink().clone();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(33));
        if is_running.load(Ordering::SeqCst) {
            life.lock().unwrap().step(1 << *jump_factor.lock().unwrap());
            sink.send(Box::new(|_: &mut cursive::Cursive| {}));
        }
    });

    siv.run();
}
