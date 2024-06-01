mod extractor;
use crate::extractor::extractor::extract;
fn main() {
    
    let image = vec![
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 1, 1, 0, 1, 1, 1, 0, 1, 0],
        vec![0, 1, 0, 0, 0, 0, 1, 0, 0, 0],
        vec![0, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 0],
        vec![0, 1, 1, 1, 0, 1, 0, 1, 1, 0],
        vec![0, 1, 0, 0, 1, 1, 0, 1, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    print_image(&image);

    let result_image = extract(&image);

    println!("Resulting image after minutia extraction:");
    print_image(&result_image);
}

fn print_image(image: &Vec<Vec<u8>>) {
    for row in image {
        for &pixel in row {
            print!("{}", if pixel == 1 { '#' } else { '.' });
        }
        println!();
    }
}
