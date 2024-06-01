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

pub fn extract(image: &Vec<Vec<u8>>) -> Vec<Vec<u8>> {
    let thinned_image = thin(image);
    let minutia = mark_minutia(&thinned_image);
    let res_image = remove_false_minutia(thinned_image, minutia, 10.0, 0.5);
    res_image
}

//pub fn run() -> bool {
//    call_fingerprint_capture();
//    let image = prep()
