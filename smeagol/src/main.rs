fn main() {
    let mut life = smeagol::Life::from_macrocell_file("./assets/waterbear.mc").unwrap();
    life.step(1);
}
