/// Thins the ridges in the fingerprint image until they are one pixel wide.
fn thin_ridges(image: &mut Vec<Vec<u8>>) {
    let mut marked_pixels = vec![vec![false; image[0].len()]; image.len()];
    let mut has_changes = true;
    
    // Iterative process to thin the ridges
    while has_changes {
        has_changes = false;
        
        // Mark redundant pixels
        mark_redundant_pixels(image, &mut marked_pixels);
        
        // Remove marked pixels
        has_changes = remove_marked_pixels(image, &marked_pixels) || has_changes;
    }
}

/// Marks redundant pixels in the fingerprint image.
fn mark_redundant_pixels(image: &Vec<Vec<u8>>, marked_pixels: &mut Vec<Vec<bool>>) {
    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            // Check if the pixel is a ridge pixel
            if image[i][j] == 1 {
                // Check for redundancy in 3x3 window
                let mut count = 0;
                for x in i - 1..=i + 1 {
                    for y in j - 1..=j + 1 {
                        if image[x][y] == 1 {
                            count += 1;
                        }
                    }
                }
                // Mark the pixel for removal if redundant
                if count > 2 && count < 8 {
                    marked_pixels[i][j] = true;
                }
            }
        }
    }
}

/// Removes marked pixels from the fingerprint image.
fn remove_marked_pixels(image: &mut Vec<Vec<u8>>, marked_pixels: &Vec<Vec<bool>>) -> bool {
    let mut has_changes = false;
    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            if marked_pixels[i][j] {
                image[i][j] = 0;
                has_changes = true;
            }
        }
    }
    has_changes
}

/// Removes H breaks from the thinned ridge map.
fn remove_h_breaks(thinned_ridge_map: &mut Vec<Vec<u8>>) {
    let mut has_changes = true;
    // Iteratively remove H breaks
    while has_changes {
        has_changes = false;
        for i in 1..thinned_ridge_map.len() - 1 {
            for j in 1..thinned_ridge_map[0].len() - 1 {
                if thinned_ridge_map[i][j] == 1 {
                    let mut h_break = true;
                    if thinned_ridge_map[i - 1][j] == 1 && thinned_ridge_map[i + 1][j] == 1 {
                        if thinned_ridge_map[i][j - 1] == 0 && thinned_ridge_map[i][j + 1] == 0 {
                            h_break = false;
                        }
                    }
                    if !h_break {
                        thinned_ridge_map[i][j] = 0;
                        has_changes = true;
                    }
                }
            }
        }
    }
}

/// Removes isolated points from the thinned ridge map.
fn remove_isolated_points(thinned_ridge_map: &mut Vec<Vec<u8>>) {
    for i in 1..thinned_ridge_map.len() - 1 {
        for j in 1..thinned_ridge_map[0].len() - 1 {
            if thinned_ridge_map[i][j] == 1 {
                let sum_neighbors = thinned_ridge_map[i - 1][j - 1] + thinned_ridge_map[i - 1][j]
                    + thinned_ridge_map[i - 1][j + 1] + thinned_ridge_map[i][j - 1]
                    + thinned_ridge_map[i][j + 1] + thinned_ridge_map[i + 1][j - 1]
                    + thinned_ridge_map[i + 1][j] + thinned_ridge_map[i + 1][j + 1];
                if sum_neighbors == 0 {
                    thinned_ridge_map[i][j] = 0;
                }
            }
        }
    }
}

/// Removes spikes from the thinned ridge map.
fn remove_spikes(thinned_ridge_map: &mut Vec<Vec<u8>>) {
    let mut has_changes = true;
    // Iteratively remove spikes
    while has_changes {
        has_changes = false;
        for i in 1..thinned_ridge_map.len() - 1 {
            for j in 1..thinned_ridge_map[0].len() - 1 {
                if thinned_ridge_map[i][j] == 1 {
                    let sum_neighbors = thinned_ridge_map[i - 1][j - 1] + thinned_ridge_map[i - 1][j]
                        + thinned_ridge_map[i - 1][j + 1] + thinned_ridge_map[i][j - 1]
                        + thinned_ridge_map[i][j + 1] + thinned_ridge_map[i + 1][j - 1]
                        + thinned_ridge_map[i + 1][j] + thinned_ridge_map[i + 1][j + 1];
                    if sum_neighbors == 1 {
                        thinned_ridge_map[i][j] = 0;
                        has_changes = true;
                    }
                }
            }
        }
    }
}

/// Calculates the crossing number for a pixel in the thinned ridge map.
fn calculate_crossing_number(image: &Vec<Vec<u8>>, i: usize, j: usize) -> u8 {
    let mut sum = 0;
    
    if image[i][j] == 0 {
        return 0; // Return 0 if the central pixel is not a ridge pixel
    }
    
    // Define the offsets for neighboring pixels in a 3x3 window
    let offsets = vec![
        (-1, -1), (-1, 0), (-1, 1),
        (0, -1),  (0, 0),  (0, 1),
        (1, -1),  (1, 0),  (1, 1),
    ];
    
    for offset in offsets {
        let x = (i as isize + offset.0) as usize;
        let y = (j as isize + offset.1) as usize;
        
        if image[x][y] == 1 {
            sum += 1;
        }
    }
    
    sum
}

/// Marks minutiae points (ridge endings and bifurcation points) in the thinned ridge map.
fn mark_minutiae_points(image: &Vec<Vec<u8>>) -> Vec<(usize, usize, &'static str)> {
    let mut minutiae_points = Vec::new();
    
    for i in 1..image.len() - 1 {
        for j in 1..image[0].len() - 1 {
            let crossing_number = calculate_crossing_number(image, i, j);
            if crossing_number == 1 {
                minutiae_points.push((i, j, "Ridge Ending"));
            } else if crossing_number == 3 {
                minutiae_points.push((i, j, "Ridge Bifurcation"));
            }
            // Handle the case when CN is 2 (just a ridge)
            // Here, we'll simply print a message indicating it's a ridge
            else if crossing_number == 2 {
                println!("Ridge at ({}, {})", i, j);
            }
        }
    }
    
    minutiae_points
}

