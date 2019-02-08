use smeagol_cli::{views, State};

fn main() {
    let (term_width, term_height) = termion::terminal_size().unwrap();
    let life = smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap();

    let mut siv = cursive::Cursive::default();

    let state = State::new_centered(life, term_width as u64, term_height as u64);

    siv.add_fullscreen_layer(views::main_view(&state));

    smeagol_cli::key::setup_key_commands(&mut siv, &state);

    smeagol_cli::start_smeagol_thread(&mut siv, &state);

    siv.run();
}
