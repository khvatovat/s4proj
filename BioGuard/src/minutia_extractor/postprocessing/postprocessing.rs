#[derive(Debug, Clone)]
pub struct Minutia {
    pub x: f64,
    pub y: f64,
    pub angle: f64,
    pub minutia_type: MinutiaType,
}

#[derive(Debug, Clone)]
pub enum MinutiaType {
    RidgeEnding,
    Bifurcation,
}

// Function to calculate the distance between two minutiae points
pub fn calculate_distance(minutia1: &Minutia, minutia2: &Minutia) -> f64 {
    ((minutia1.x - minutia2.x).powi(2) + (minutia1.y - minutia2.y).powi(2)).sqrt()
}

            // Mark `from` as empty
// Function to calculate the orientation angle between two minutiae points
pub fn calculate_orientation_angle(minutia1: &Minutia, minutia2: &Minutia) -> f64 {
    (minutia2.y - minutia1.y).atan2(minutia2.x - minutia1.x)
}

// Function to remove false minutiae based on modified fuzzy rules
pub fn remove_false_minutiae(mutiae: &mut Vec<Minutia>, d: f64) {
    let mut i = 0;
    while i < mutiae.len() {
        let mut j = i + 1;
        while j < mutiae.len() {
            let distance = calculate_distance(&mutiae[i], &mutiae[j]);
            let minutia_type_i = mutiae[i].minutia_type.clone(); // Clone the MinutiaType
            let minutia_type_j = mutiae[j].minutia_type.clone(); // Clone the MinutiaType
            if distance < d {
                match (minutia_type_i, minutia_type_j) {
                    (MinutiaType::RidgeEnding, MinutiaType::RidgeEnding)
                    | (MinutiaType::Bifurcation, MinutiaType::Bifurcation)
                    | (MinutiaType::RidgeEnding, MinutiaType::Bifurcation)
                    | (MinutiaType::Bifurcation, MinutiaType::RidgeEnding) => {
                        mutiae.remove(j);
                        mutiae.remove(i);
                        continue;
                    }
   //                 _ => {}
                }
            }
            let angle = calculate_orientation_angle(&mutiae[i], &mutiae[j]);
            // Assuming the angle variation threshold is small, for example, 0.1 radians
            if angle.abs() < 0.1 {
                let mut k = 0;
                let mut found_between = false;
                while k < mutiae.len() {
                    if k != i && k != j {
                        let dist1 = calculate_distance(&mutiae[i], &mutiae[k]);
                        let dist2 = calculate_distance(&mutiae[j], &mutiae[k]);
                        if dist1 < distance && dist2 < distance {
                            found_between = true;
                            break;
                        }
                    }
                    k += 1;
                }
                if !found_between {
                    mutiae.remove(j);
                    mutiae.remove(i);
                    continue;
                }
            }
            j += 1;
        }
        i += 1;
    }
}
