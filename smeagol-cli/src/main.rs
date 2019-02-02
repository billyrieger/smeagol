const DEFAULT_NUM_STEPS: &'static str = "10";
const DEFAULT_STEP_SIZE: &'static str = "1024";

fn main() {
    let app = clap::App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::with_name("input")
                .short("i")
                .long("input")
                .help("The pattern to simulate.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            clap::Arg::with_name("step-size")
                .short("s")
                .long("step-size")
                .help("The number of generations to advance each step.")
                .takes_value(true),
        )
        .arg(
            clap::Arg::with_name("num-steps")
                .short("n")
                .long("num-steps")
                .help("The number of steps to take.")
                .takes_value(true),
        );

    let matches = app.get_matches();
    let mut life = smeagol::Life::from_rle_file(matches.value_of("input").unwrap())
        .expect("could not load RLE file");
    println!("loaded");
    let step_size = matches
        .value_of("step-size")
        .or(Some(DEFAULT_STEP_SIZE))
        .and_then(|n_str| n_str.parse().ok())
        .expect("could not parse step-size");
    let num_steps = matches
        .value_of("num-steps")
        .or(Some(DEFAULT_NUM_STEPS))
        .and_then(|n_str| n_str.parse().ok())
        .expect("could not parse num-steps");
    for _ in 0..num_steps {
        life.step(step_size);
        println!("{}\n{}\n", life.generation(), life.population());
    }
}
