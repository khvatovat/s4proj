use BioGuard::minutia_extractor::preprocessing::preprocessing::load_image;
fn main() {
    let path= String::from("..\\..\\Fingerprint\\data\\fingerprint_1.bmp");
    load_image(path);
    println!("Hello, world!");
}
