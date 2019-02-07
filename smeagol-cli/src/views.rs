use crate::State;
use std::sync::{Arc, Mutex};

struct GenerationView {
    life: Arc<Mutex<smeagol::Life>>,
}

impl GenerationView {
    pub fn new(life: Arc<Mutex<smeagol::Life>>) -> Self {
        Self { life }
    }

    fn format(&self) -> String {
        format!("gen: {}", self.life.lock().unwrap().generation())
    }
}

impl cursive::view::View for GenerationView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print((0, 0), &self.format());
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        // (width, height)
        (self.format().len(), 1).into()
    }
}

struct PopulationView {
    life: Arc<Mutex<smeagol::Life>>,
}

impl PopulationView {
    pub fn new(life: Arc<Mutex<smeagol::Life>>) -> Self {
        Self { life }
    }

    fn format(&self) -> String {
        format!("pop: {}", self.life.lock().unwrap().population())
    }
}

impl cursive::view::View for PopulationView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print((0, 0), &self.format());
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        // (width, height)
        (self.format().len(), 1).into()
    }
}

struct StepView {
    step: Arc<Mutex<u64>>,
}

impl StepView {
    fn new(step: Arc<Mutex<u64>>) -> Self {
        Self { step }
    }

    fn format(&self) -> String {
        format!("step: {}", self.step.lock().unwrap())
    }
}

impl cursive::view::View for StepView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print((0, 0), &self.format());
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        // (width, height)
        (self.format().len(), 1).into()
    }
}

struct CenterView {
    center: Arc<Mutex<(i64, i64)>>,
}

impl CenterView {
    fn new(center: Arc<Mutex<(i64, i64)>>) -> Self {
        Self { center }
    }

    fn format(&self) -> String {
        let center = self.center.lock().unwrap();
        format!("center: ({}, {})", center.0, center.1)
    }
}

impl cursive::view::View for CenterView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print((0, 0), &self.format());
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        // (width, height)
        (self.format().len(), 1).into()
    }
}

struct ScaleView {
    scale: Arc<Mutex<u64>>,
}

impl ScaleView {
    fn new(scale: Arc<Mutex<u64>>) -> Self {
        Self { scale }
    }

    fn format(&self) -> String {
        format!("scale: {}:1", self.scale.lock().unwrap())
    }
}

impl cursive::view::View for ScaleView {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print((0, 0), &self.format());
    }

    fn required_size(&mut self, _: cursive::vec::Vec2) -> cursive::vec::Vec2 {
        // (width, height)
        (self.format().len(), 1).into()
    }
}

pub struct LifeView {
    life: Arc<Mutex<smeagol::Life>>,
    center: Arc<Mutex<(i64, i64)>>,
    scale: Arc<Mutex<u64>>,
}

impl LifeView {
    fn new(
        life: Arc<Mutex<smeagol::Life>>,
        center: Arc<Mutex<(i64, i64)>>,
        scale: Arc<Mutex<u64>>,
    ) -> Self {
        Self {
            life,
            center,
            scale,
        }
    }
}

impl cursive::view::View for LifeView {
    #[allow(clippy::many_single_char_names)]
    fn draw(&self, printer: &cursive::Printer) {
        let width = printer.output_size.x as i64;
        let height = printer.output_size.y as i64;
        let front_color = cursive::theme::Color::Rgb(255, 255, 255);
        let back_color = cursive::theme::Color::Rgb(0, 0, 0);
        let mut life = self.life.lock().unwrap();
        let zoom_factor = *self.scale.lock().unwrap() as i64;
        let zoom_factor_minus_1 = zoom_factor - 1;
        let center = self.center.lock().unwrap();
        for x in 0..width {
            for y in 0..height {
                printer.with_color(
                    cursive::theme::ColorStyle::new(front_color, back_color),
                    |printer| {
                        printer.print((x as u32, y as u32), {
                            let x_offset = 2 * (x - (width / 2)) * zoom_factor + center.0;
                            let y_offset = 4 * (y - (height / 2)) * zoom_factor + center.1;
                            // +---+---+
                            // | a | b |
                            // +---+---+
                            // | c | d |
                            // +---+---+
                            // | e | f |
                            // +---+---+
                            // | g | h |
                            // +---+---+
                            let a = if life.contains_alive_cells(
                                (x_offset, y_offset),
                                (
                                    x_offset + zoom_factor_minus_1,
                                    y_offset + zoom_factor_minus_1,
                                ),
                            ) {
                                1
                            } else {
                                0
                            };
                            let b = if life.contains_alive_cells(
                                (x_offset + zoom_factor, y_offset),
                                (
                                    x_offset + zoom_factor + zoom_factor_minus_1,
                                    y_offset + zoom_factor_minus_1,
                                ),
                            ) {
                                1
                            } else {
                                0
                            };
                            let c = if life.contains_alive_cells(
                                (x_offset, y_offset + zoom_factor),
                                (
                                    x_offset + zoom_factor_minus_1,
                                    y_offset + zoom_factor + zoom_factor_minus_1,
                                ),
                            ) {
                                1
                            } else {
                                0
                            };
                            let d = if life.contains_alive_cells(
                                (x_offset + zoom_factor, y_offset + zoom_factor),
                                (
                                    x_offset + zoom_factor + zoom_factor_minus_1,
                                    y_offset + zoom_factor + zoom_factor_minus_1,
                                ),
                            ) {
                                1
                            } else {
                                0
                            };
                            let e = if life.contains_alive_cells(
                                (x_offset, y_offset + 2 * zoom_factor),
                                (
                                    x_offset + zoom_factor_minus_1,
                                    y_offset + 2 * zoom_factor + zoom_factor_minus_1,
                                ),
                            ) {
                                1
                            } else {
                                0
                            };
                            let f = if life.contains_alive_cells(
                                (x_offset + zoom_factor, y_offset + 2 * zoom_factor),
                                (
                                    x_offset + zoom_factor + zoom_factor_minus_1,
                                    y_offset + 2 * zoom_factor + zoom_factor_minus_1,
                                ),
                            ) {
                                1
                            } else {
                                0
                            };
                            let g = if life.contains_alive_cells(
                                (x_offset, y_offset + 3 * zoom_factor),
                                (
                                    x_offset + zoom_factor_minus_1,
                                    y_offset + 3 * zoom_factor + zoom_factor_minus_1,
                                ),
                            ) {
                                1
                            } else {
                                0
                            };
                            let h = if life.contains_alive_cells(
                                (x_offset + zoom_factor, y_offset + 3 * zoom_factor),
                                (
                                    x_offset + zoom_factor + zoom_factor_minus_1,
                                    y_offset + 3 * zoom_factor + zoom_factor_minus_1,
                                ),
                            ) {
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
        let scale = *self.scale.lock().unwrap() as i64;
        let mut center = self.center.lock().unwrap();
        match event {
            cursive::event::Event::Char('H') => {
                center.0 -= scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Char('L') => {
                center.0 += scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Char('K') => {
                center.1 -= scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Char('J') => {
                center.1 += scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Char('h') => {
                center.0 -= 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Char('l') => {
                center.0 += 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Char('k') => {
                center.1 -= 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Char('j') => {
                center.1 += 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Key(cursive::event::Key::Left) => {
                center.0 -= 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Key(cursive::event::Key::Right) => {
                center.0 += 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Key(cursive::event::Key::Up) => {
                center.1 -= 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            cursive::event::Event::Key(cursive::event::Key::Down) => {
                center.1 += 4 * scale;
                cursive::event::EventResult::Consumed(None)
            }
            _ => cursive::event::EventResult::Ignored,
        }
    }
}

pub fn main_view(state: &State) -> cursive::views::LinearLayout {
    let padding = ((1, 1), (0, 0));
    cursive::views::LinearLayout::vertical()
        .child(cursive::view::Boxable::full_screen(LifeView::new(
            state.life.clone(),
            state.center.clone(),
            state.scale.clone(),
        )))
        .child(
            cursive::views::LinearLayout::horizontal()
                .child(cursive::views::PaddedView::new(
                    padding,
                    GenerationView::new(state.life.clone()),
                ))
                .child(cursive::views::TextView::new("|"))
                .child(cursive::views::PaddedView::new(
                    padding,
                    PopulationView::new(state.life.clone()),
                ))
                .child(cursive::views::TextView::new("|"))
                .child(cursive::views::PaddedView::new(
                    padding,
                    StepView::new(state.step.clone()),
                ))
                .child(cursive::view::Boxable::full_width(
                    cursive::views::DummyView,
                ))
                .child(cursive::views::PaddedView::new(
                    padding,
                    CenterView::new(state.center.clone()),
                ))
                .child(cursive::views::TextView::new("|"))
                .child(cursive::views::PaddedView::new(
                    padding,
                    ScaleView::new(state.scale.clone()),
                )),
        )
}
