fn main() {
    let mut life = smeagol::Life::from_rle_file(std::env::args().nth(1).unwrap()).unwrap();
    println!("{}", life.generation());
    println!("{}", life.population());
    for _ in 0..100 {
        life.step_pow_2(15);
        println!("{}", life.generation());
        println!("{}", life.population());
    }
}
