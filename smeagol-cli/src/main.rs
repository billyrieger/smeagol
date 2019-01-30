fn main() {
    let mut life = smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap();
    println!("loaded");
    for _ in 0..20 {
        life.step(
            std::env::args()
                .nth(2)
                .and_then(|n_str| n_str.parse().ok())
                .unwrap_or(1000),
        );
        println!("{}", life.generation());
    }
    // life.save_png("out.png");
}
