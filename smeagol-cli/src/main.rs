use std::{
    sync::{atomic::Ordering, Arc, Mutex},
};

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
        let mut alive_coords = std::collections::HashSet::new();
        alive_coords.extend(self.life.lock().unwrap().get_alive_cells());
        for x in 0..width {
            for y in 0..height {
                printer.with_color(
                    cursive::theme::ColorStyle::new(front_color, back_color),
                    |printer| {
                        printer.print((x as u32, y as u32), {
                            let x_offset = 2 * (x as i64 + self.center.0 - (width / 2) as i64);
                            let y_offset = 4 * (y as i64 + self.center.1 - (height / 2) as i64);
                            let a = if alive_coords.contains(&(x_offset, y_offset)) {
                                1
                            } else {
                                0
                            };
                            let b = if alive_coords.contains(&(x_offset + 1, y_offset)) {
                                1
                            } else {
                                0
                            };
                            let c = if alive_coords.contains(&(x_offset, y_offset + 1)) {
                                1
                            } else {
                                0
                            };
                            let d = if alive_coords.contains(&(x_offset + 1, y_offset + 1)) {
                                1
                            } else {
                                0
                            };
                            let e = if alive_coords.contains(&(x_offset, y_offset + 2)) {
                                1
                            } else {
                                0
                            };
                            let f = if alive_coords.contains(&(x_offset + 1, y_offset + 2)) {
                                1
                            } else {
                                0
                            };
                            let g = if alive_coords.contains(&(x_offset, y_offset + 3)) {
                                1
                            } else {
                                0
                            };
                            let h = if alive_coords.contains(&(x_offset + 1, y_offset + 3)) {
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
    let is_running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    siv.add_fullscreen_layer(cursive::view::Boxable::full_screen(LifeView::new(
        life.clone(),
    )));
    siv.add_global_callback(' ', enclose!((is_running) move |_| is_running.store(!is_running.load(Ordering::SeqCst), Ordering::SeqCst)));
    // siv.add_global_callback(
    //     cursive::event::Key::Tab,
    //     enclose!((life) move |_| life.lock().step(32)),
    // );
    siv.set_fps(30);
    let sink = siv.cb_sink().clone();

    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(33));
        if is_running.load(Ordering::SeqCst) {
            life.lock().unwrap().step(1);
            sink.send(Box::new(|_: &mut cursive::Cursive| {}));
        }
    });

    siv.run();
}
