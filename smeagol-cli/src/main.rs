fn main() {
    let mut life = smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap();
    println!("loaded");
    for _ in 0..20 {
        life.step(35328);
        println!("{}", life.generation());
    }
    // life.save_png("out.png");
}
