use BioGuard::minutia_extractor::preprocessing::preprocessing::load_image;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
fn main() {
    let path= String::from("..\\..\\Fingerprint\\data\\fingerprint_1.bmp");
    load_image(path);
    println!("Hello, world!");
}
