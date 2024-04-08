//use std::io::{self, Write};
mod minutia_extractor;
//use crate::minutia_extractor::postprocessing::postprocessing;
//use minutia_extractor::postprocessing::{
//    Minutia,
//    MinutiaType,
//    postprocessing::{
//        calculate_distance,
//        calculate_orientation_angle,
//        remove_false_minutiae,
//    },
//};
use crate::minutia_extractor::postprocessing::postprocessing;
//use crate::minutia_extractor::postprocessing::postprocessing::Minutia;
use crate::minutia_extractor::postprocessing::postprocessing::MinutiaType;

//fn print_matrix(matrix: &Vec<Vec<u8>>) {
//    let width = matrix[0].len();
//
//    // Print column indices
//    print!("  ");
//    for i in 0..width {
//        print!("{:^3}", i);
//    }
//    println!();
//
//    // Print top border
//    println!("  ┌{}┐", "─".repeat(width * 3));
//
//    for (i, row) in matrix.iter().enumerate() {
//        // Print row index
//        print!("{:<2}│", i);
//
//        for &cell in row {
//            // Print the appropriate character based on the cell value
//            match cell {
//                0 => print!("   "),
//                1 => print!(" █ "),
//                _ => print!(" X "),
//            }
//        }
//
//        // Print row index
//        println!("│{}", i);
//    }
//
//    // Print bottom border
//    println!("  └{}┘", "─".repeat(width * 3));
//
//    // Print column indices
//    print!("  ");
//    for i in 0..width {
//        print!("{:^3}", i);
//    }
//    println!();
//}
//
fn main() {
    //let mut fingerprint_image = vec![
    //vec![0, 0, 1, 1, 1, 0, 0, 0, 0, 0],
    //vec![0, 1, 1, 0, 0, 1, 0, 0, 0, 0],
    //vec![0, 1, 0, 0, 0, 1, 0, 0, 0, 0],
    //vec![0, 0, 1, 0, 0, 1, 0, 0, 0, 0],
    //vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
    //vec![0, 1, 1, 0, 1, 1, 0, 0, 0, 0],
    //vec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0],
    //vec![0, 0, 0, 0, 1, 1, 0, 1, 0, 0],
    //vec![0, 0, 0, 1, 0, 0, 0, 1, 0, 0],
    //vec![0, 1, 1, 1, 0, 0, 0, 0, 0, 1],
    //vec![0, 0, 0, 1, 1, 0, 0, 0, 0, 0],
    //vec![0, 0, 0, 0, 0, 1, 1, 0, 0, 0],
    //vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    //vec![0, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    //];
    //// Print the fingerprint matrix
    //println!("Fingerprint Matrix:");
    //print_matrix(&fingerprint_image);

    //// Prompt the user to input coordinates of a pixel
    //let mut input_coords = String::new();
    //print!("Please input the coordinates of a pixel (x y): ");
    //io::stdout().flush().unwrap();
    //io::stdin().read_line(&mut input_coords).expect("Failed to read line");

    //let coords: Vec<usize> = input_coords
    //    .trim()
    //    .split_whitespace()
    //    .map(|num| num.parse().expect("Invalid input"))
    //    .collect();

    //let x = coords[0];
    //let y = coords[1];
    //// Thin the ridges
    ////thin_ridges(&mut fingerprint_image);

    //// Apply morphological operations
    ////remove_h_breaks(&mut fingerprint_image);
    ////remove isolated_points(&mut fingerprint_image);
    ////remove_spikes(&mut fingerprint_image);

    //// Find minutiae points
    ////let minutiae_points = mark_minutiae_points(&fingerprint_image);
    //fingerprint_image[x][y] = 2;
    //print_matrix(&fingerprint_image);
    //match x {
    //        6 => println!("Pixel at ({}, {}) is a bifurcation point.", x, y),
    //        13 => println!("Pixel at ({}, {}) is a termination point.", x, y),
    //        _ => println!("Pixel at ({}, {}) is a ridge point.", x, y),
    //    }
    let mut minutiae = vec![
        postprocessing::Minutia {
            x: 10.0,
            y: 20.0,
            angle: 0.0,
            minutia_type: MinutiaType::RidgeEnding,
        },
        postprocessing::Minutia {
            x: 15.0,
            y: 25.0,
            angle: 0.0,
            minutia_type: MinutiaType::Bifurcation,
        },
        // Add more minutiae as needed
    ];

    let d = 5.0; // Threshold distance
    postprocessing::remove_false_minutiae(&mut minutiae, d);

    println!("Removal complete");
}

