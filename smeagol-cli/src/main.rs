fn main() {
    let mut life = smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap();
    println!("loaded");
    life.step_pow_2(std::env::args().nth(2).and_then(|n_str| n_str.parse().ok()).unwrap_or(10));
    println!("stepped");
    life.save_png("out.png");
}
