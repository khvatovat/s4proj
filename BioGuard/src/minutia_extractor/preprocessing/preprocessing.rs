use image::{open, GenericImage, GenericImageView, ImageBuffer};

pub fn load_image(path: String)
{
    let tmp_img = open(path).unwrap();
    println!("Ta maman");
} 

