#[derive(Debug, Clone, PartialEq)]
pub enum MinutiaType {
    RidgeEnding,
    Bifurcation,
}

#[derive(Debug, Clone)]
pub struct Minutia {
    x: usize,
    y: usize,
    minutia_type: MinutiaType,
}

use std::path::Path;


///PREPROCESSING BLOCK
///
// Histogram equalization
fn histogram_equalization(image_path: &str) -> Vec<Vec<u8>> {
    let img = image::open(&Path::new(image_path)).unwrap().to_luma8();
    let mut histogram = [0u32; 256];
    let mut cdf = [0u32; 256];
    let mut res = vec![vec![0u8; img.width() as usize]; img.height() as usize];

    for pixel in img.pixels() {
        histogram[pixel[0] as usize] += 1;
    }

    // Calculate the cumulative distribution function (CDF)
    cdf[0] = histogram[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + histogram[i];
    }

    //let mut image = vec![vec![0u8; img.width() as usize]; img.height() as usize];

    let n = img.width() * img.height();
    let scale = 255.0 / (n as f32);
    //for (y, row) in image.iter_mut().enumerate() {
    for (y, row) in res.iter_mut().enumerate() {
        for (x, pixel) in row.iter_mut().enumerate() {
            let orig = img.get_pixel(x as u32, y as u32)[0];
            *pixel = (cdf[orig as usize] as f32 * scale).round() as u8;
        }
    }
    res
}

// To Black&Whrite
fn binarization(input: Vec<Vec<u8>>, threshold: u8) -> Vec<Vec<u8>> {
    let mut res = vec![vec![0u8; input[0].len()]; input.len()];
    for (y, row) in input.iter().enumerate() {
        for (x, &pixel) in row.iter().enumerate() {
            res[y][x] = if pixel > threshold { 1 } else { 0 };
        }
    }
    res
}
///
/// MATIS' PART
   // // Calculate histogram
   // let mut histogram = [0u8; 256];
   // for Sub in image {
   //     for pixel in Sub {
   //         let intensity = pixel as usize;
   //         histogram[intensity] += 1;
   //     }
   // }

   // // Calculate cumulative distribution function (CDF)
   // let mut cdf = [0u8; 256];
   // let mut cumulative_sum = 0;
   // for (intensity, &count) in histogram.iter().enumerate() {
   //     cumulative_sum += count;
   //     cdf[intensity] = cumulative_sum;
   // }

   // // Normalize CDF
   // let total_pixels = image.len() * image[0].len();
   // let cdf_normalized: Vec<u8> = cdf.iter().map(|&value| ((value as f32 / total_pixels as f32) * 255.0) as u8).collect();

   // // Apply histogram equalization
   // let mut equalized_image = Vec::new();
   // let mut sub = Vec::new();
   // for i in 0..image.len() {
   //     sub.clear();
   //     for j in 0..image[0].len() {
   //         let intensity = image[i][j] as usize;
   //         let equalized_intensity = cdf_normalized[intensity] as u8;
   //         sub.push(equalized_intensity);
   //     }
   //     equalized_image.push(sub);
   // }

   // equalized_image


//// subfunction of black and white
//fn otsu_threshold(image: Vec<Vec<u8>>) -> u8 {
//    let total_pixels = (image.len() * image[0].len()) as u32;
//    let hist = calculate_histogram(image);
//    let mut sum = 0.0;
//
//    for (i, &count) in hist.iter().enumerate() {
//        sum += i as f64 * count as f64;
//    }
//
//    let mut sum_b = 0.0;
//    let mut w_b = 0;
//    let mut w_f;
//
//    let mut var_max = 0.0;
//    let mut threshold = 0;
//
//    for (i, &count) in hist.iter().enumerate() {
//        w_b += count;
//        if w_b == 0 {
//            continue;
//        }
//        w_f = total_pixels - w_b;
//        if w_f == 0 {
//            break;
//        }
//        sum_b += i as f64 * count as f64;
//        let mean_b = sum_b / w_b as f64;
//        let mean_f = (sum - sum_b) / w_f as f64;
//
//        let var_between = w_b as f64 * w_f as f64 * (mean_b - mean_f) * (mean_b - mean_f);
//
//        if var_between > var_max {
//            var_max = var_between;
//            threshold = i;
//        }
//    }
//
//    threshold as u8
//}
//
//// subfunction of black and white
//fn calculate_histogram(image: Vec<Vec<u8>>) -> [u32; 256] {
//    let mut histogram = [0u32; 256];
//
//    for Sub in image {
//        for pixel in Sub {
//            let intensity = pixel as usize;
//            histogram[intensity] += 1;
//	}
//    }
//
//    histogram
//}
//
//// Black and White
//fn binarize_image(image: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
//    let threshold = otsu_threshold(image);
//    let mut binary_image = Vec::new();
//    let mut sub = Vec::new();
//    for i in 0..image.len() {
//        sub.clear();
//        for j in 0..image[0].len() {
//            if image[i][j] > threshold {
//                sub.push(255);
//            }
//            else {
//               sub.push(0);
//            }
//        }
//        binary_image.push(sub);
//    }
//
//
//    binary_image
//}
//
//// Fourier
//// Function to perform Fourier transformation on a grayscale image
//fn fourier_transform(image: Vec<Vec<u8>>) -> Vec<Vec<u8>> {
//    // Perform Fourier transform (forward)
//    let mut freq_domain = rustfft::FftPlanner::new().plan_fft_forward(image.len() as usize * image[0].len() as usize);
//    let mut complex_image: Vec<Complex<f64>> = Vec::new();
//    for sub in image {
//        for pixel in sub {
//            complex_image.push(Complex::new(pixel as f64, 0.0))
//        }
//    }
//    freq_domain.process(&mut complex_image);
//
//    // Perform inverse Fourier transform
//    let mut inv_freq_domain = rustfft::FftPlanner::new().plan_fft_inverse(image.len() as usize * image[0].len() as usize);
//    inv_freq_domain.process(&mut complex_image);
//    
//    // Normalize and create the output image
//    let max_intensity = complex_image.iter().map(|c| c.norm()).fold(0.0, f64::max);
//    let scaled_image: Vec<u8> = complex_image.iter().map(|c| (255.0 * c.norm() / max_intensity) as u8).collect();
//    
//    let mut dynamicimage = Vec::new();
//    let mut sub = Vec::new();
//    let mut it = scaled_image.iter();
//    let l = image[0].len();
//    for i in 0..image.len() {
//        sub.clear();
//        for j in 0..l {
//            sub.push(it.next().unwrap());
//        }
//        dynamicimage.push(sub);
//    }
//
//    dynamicimage
///END OF MATIS' PART
///

///
///END OF PREPROCESSING BLOCK

///EXTRACTION BLOCK
///
//Implements thhe Zhang-Suen thinning algorithm (https://dl.acm.org/doi/epdf/10.1145/357994.358023)
//and applies the 3 morphological operations after the thinngin
fn thin(image: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let mut image = image.clone();
    let mut changed = true;
    
    while changed {
        changed = false;
        let mut to_del = vec![vec![false; image[0].len()]; image.len()];
        
        // Step 1
        for i in 1..image.len() - 1 {
            for j in 1..image[0].len() - 1 {
                if image[i][j] == 1 && step1(&image, i, j) {
                    to_del[i][j] = true;
                    changed = true;
                }
            }
        }
        //Remove pixels marked in Step 1 
        for i in 0..image.len() {
            for j in 0..image[0].len() {
                if to_del[i][j] {
                    image[i][j] = 0;
                }
            }
        }
        
        // Step 2
        for i in 1..image.len() - 1 {
            for j in 1..image[0].len() - 1 {
                if image[i][j] == 1 && step2(&image, i, j) {
                    to_del[i][j] = true;
                    changed = true;
                }
            }
        }
        //Remove pixels marked in Strp 2
        for i in 0..image.len() {
            for j in 0..image[0].len() {
                if to_del[i][j] {
                    image[i][j] = 0;
                }
            }
        }
    }
   
    remove_h_breaks(&mut image);
    remove_isolated_points(&mut image);
    remove_spikes(&mut image);

    image
}

//Applies the first step conditions of the algothithm
fn step1(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    let transition_count = count_transitions(&neighbors);
    let neighbor_count = neighbors.iter().sum::<u8>();
    
    neighbor_count >= 2 && neighbor_count <= 6 &&
    transition_count == 1 &&
    neighbors[0] * neighbors[2] * neighbors[4] == 0 &&
    neighbors[2] * neighbors[4] * neighbors[6] == 0
}
//Applies the second step conditions og the algorithm
fn step2(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    let transition_count = count_transitions(&neighbors);
    let neighbor_count = neighbors.iter().sum::<u8>();
    
    neighbor_count >= 2 && neighbor_count <= 6 &&
    transition_count == 1 &&
    neighbors[0] * neighbors[2] * neighbors[6] == 0 &&
    neighbors[0] * neighbors[4] * neighbors[6] == 0
}

//Returns a vector with the 8 neighbours in the specified order
fn get_neighbors(image: &Vec<Vec<u8>>, x: usize, y: usize) -> Vec<u8> {
    vec![
        image[x - 1][y],     // N
        image[x - 1][y + 1], // NE
        image[x][y + 1],     // E
        image[x + 1][y + 1], // SE
        image[x + 1][y],     // S
        image[x + 1][y - 1], // SW
        image[x][y - 1],     // W
        image[x - 1][y - 1], // NW
    ]
}

//Counts the number of transitions from 0 to 1 (0 is directly followed by 1)
fn count_transitions(neighbors: &Vec<u8>) -> usize {
    let mut count = 0;
    for i in 0..neighbors.len() {
        if neighbors[i] == 0 && neighbors[(i + 1) % neighbors.len()] == 1 {
            count += 1;
        }
    }
    count
}

//Desides whether the pixel is part of an H break by comparing in to two known neighbour H-patterns
fn is_h_break(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);

    let reference = vec![
        vec![1, 0, 1, 0, 1, 1, 1, 0], // H pattern
        vec![1, 1, 1, 0, 1, 0, 1, 0], // Rotated H pattern
    ];

    reference.iter().any(|r| {
        neighbors.iter().zip(r.iter()).all(|(x, y)| x == y)
    })
}
//Morphological operation to remove the H breaks
fn remove_h_breaks(image: &mut Vec<Vec<u8>>) {
    let mut changed = true;

    while changed {
        changed = false;
        let mut to_del = vec![vec![false; image[0].len()]; image.len()];

        for i in 1..image.len() - 1 {
            for j in 1..image[0].len() - 1 {
                if is_h_break(&image, i, j) {
                    to_del[i][j] = true;
                    changed = true;
                }
            }
        }

        for i in 0..image.len() {
            for j in 0..image[0].len() {
                if to_del[i][j] {
                    image[i][j] = 0;
                }
            }
        }
    }
}

//Removes the isolated pixels
fn remove_isolated_points(image: &mut Vec<Vec<u8>>) {
    let mut to_del = vec![vec![false; image[0].len()]; image.len()];

    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            if image[i][j] == 1 && is_isolated(&image, i, j) {
                to_del[i][j] = true;
            }
        }
    }

    for i in 0..image.len() {
        for j in 0..image[0].len() {
            if to_del[i][j] {
                image[i][j] = 0;
            }
        }
    }
}
//Isolated == all the neighbouring pixels are 0
fn is_isolated(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    neighbors.iter().sum::<u8>() == 0
}

//Removes spikes
fn remove_spikes(image: &mut Vec<Vec<u8>>) {
    let mut to_del = vec![vec![false; image[0].len()]; image.len()];

    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            if image[i][j] == 1 && is_spike(&image, i, j) {
                to_del[i][j] = true;
            }
        }
    }

    for i in 0..image.len() {
        for j in 0..image[0].len() {
            if to_del[i][j] {
                image[i][j] = 0;
            }
        }
    }
}

//Pixel is a spike == it has exactly one ridge pixel in its vacinity
fn is_spike(image: &Vec<Vec<u8>>, x: usize, y: usize) -> bool {
    let neighbors = get_neighbors(image, x, y);
    neighbors.iter().sum::<u8>() == 1
}
///
///END OF EXTRACTION BLOCK


///POSTPROCESSING BLOCK
///
//Marks each valuable pixel (not a regular ridge) as either an ending, or a bifurcation point.
//Stores this information in a structure and returns a vector of valuale minutia
fn mark_minutia(image: &Vec<Vec<u8>>) -> Vec<Minutia> {
    let mut res = Vec::new();

    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            if image[i][j] == 1 {
                let neighbors = get_neighbors(image, i, j);
                let neighbor_count = neighbors.iter().sum::<u8>();

                if neighbor_count == 1 {
                    res.push(Minutia {
                        x: i,
                        y: j,
                        minutia_type: MinutiaType::RidgeEnding,
                    });
                } else if neighbor_count == 3 {
                    res.push(Minutia {
                        x: i,
                        y: j,
                        minutia_type: MinutiaType::Bifurcation,
                    });
                }
            }
        }
    }
    //println!("size: {}, {}", image.len(), image[0].len());
    res
}

fn euclidean_distance(a: &Minutia, b: &Minutia) -> f64 {
    (((a.x as f64 - b.x as f64).powi(2) + (a.y as f64 - b.y as f64).powi(2)) as f64).sqrt()
}
fn calculate_angle(a: &Minutia, b: &Minutia) -> f64 {
    let delta_x = b.x as f64 - a.x as f64;
    let delta_y = b.y as f64 - a.y as f64;
    delta_y.atan2(delta_x).abs()
}
//Removes the false minutia using Fuzzy rules
fn remove_false_minutia(mut image: Vec<Vec<u8>>, minutia: Vec<Minutia>, distance_threshold: f64, angle_threshold: f64) -> Vec<Vec<u8>> {
    let mut to_del = Vec::new();
    for i in 0..minutia.len() {
        for j in (i + 1)..minutia.len() {
            let mut is_false = false;

            let distance = euclidean_distance(&minutia[i], &minutia[j]);

            // Rule 1: Termination and Bifurcation on the same ridge
            if distance < distance_threshold
                && minutia[i].minutia_type != minutia[j].minutia_type
            {
                is_false = true;
            }

            // Rule 2: Distance between two bifurcations on the same ridge
            if !is_false
                && distance < distance_threshold
                && minutia[i].minutia_type == MinutiaType::Bifurcation
                && minutia[j].minutia_type == MinutiaType::Bifurcation
            {
                is_false = true;
            }

            // Rule 3: Distance between two terminations on the same ridge
            if !is_false
                && distance < distance_threshold
                && minutia[i].minutia_type == MinutiaType::RidgeEnding
                && minutia[j].minutia_type == MinutiaType::RidgeEnding
            {
                is_false = true;
            }

            // Rule 4: Angle
            if !is_false
                && distance < distance_threshold
                && minutia[i].minutia_type == MinutiaType::RidgeEnding
                && minutia[j].minutia_type == MinutiaType::RidgeEnding
            {
                let angle_variation = calculate_angle(&minutia[i], &minutia[j]);

                if angle_variation < angle_threshold {
                    is_false = true;
                }
            }

            if is_false {
                to_del.push(minutia[i].clone());
                to_del.push(minutia[j].clone());
            }
        }
    }

    for m in &to_del {
        image[m.x][m.y] = 0;
    }

    image
}
///
///END OF POSTPROCESSING BLOCK




pub fn extract() -> Vec<Vec<u8>> {
    //call_fingerprint_capture(); //TODO: add an import for this depending on where you have the
                                //function, the project must have only one main.rs
    let image_path = "src/input.bmp"; //TODO: correct the filename
    let hist = histogram_equalization(image_path);
    let bin = binarization(hist.clone(), 128);
    let thin = thin(&bin);
    let minutia = mark_minutia(&thin);
    let res_image = remove_false_minutia(thin, minutia, 10.0, 0.5);
    //let result = match_test(reference, res_image);//TODO: get the referense matrix, i am not sure how it
                                                  //works with the db
    //result
    res_image
}

