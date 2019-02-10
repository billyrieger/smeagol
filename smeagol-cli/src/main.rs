enum Error {
    TermSizeUnavailable,
    NoInputFile,
    CannotReadFile,
}

fn run(siv: &mut cursive::Cursive) -> Result<(), Error> {
    let (term_width, term_height) = term_size::dimensions().ok_or(Error::TermSizeUnavailable)?;
    let life = smeagol::Life::from_rle_file(std::env::args().nth(1).ok_or(Error::NoInputFile)?)
        .map_err(|_| Error::CannotReadFile)?;

    let state = smeagol_cli::State::new_centered(life, term_width as u64, (term_height - 1) as u64);

    smeagol_cli::views::add_main_view(siv, &state);
    smeagol_cli::key::setup_key_commands(siv, &state);
    smeagol_cli::start_smeagol_thread(siv, &state);

    siv.run();

    Ok(())
}

fn main() {
    if let Err(err) = {
        let mut siv = cursive::Cursive::default();
        run(&mut siv)
    } {
        match err {
            Error::TermSizeUnavailable => eprintln!("error: cannot get terminal size"),
            Error::NoInputFile => eprintln!("error: no input file given"),
            Error::CannotReadFile => eprintln!("error: cannot read file"),
        }
        std::process::exit(1);
    }
}
