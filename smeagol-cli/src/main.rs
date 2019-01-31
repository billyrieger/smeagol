fn main() {
    let mut life = smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap();
    println!("loaded");
    let step = std::env::args()
                .nth(2)
                .and_then(|n_str| n_str.parse().ok())
                .unwrap_or(1024);
    loop {
        life.step(step);
        println!("{}\n{}\n", life.generation(), life.population());
    }
}
