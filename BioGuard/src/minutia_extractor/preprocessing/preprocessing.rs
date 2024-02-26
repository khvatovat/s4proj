use image::{DynamicImage, Rgba, RgbaImage};

fn histogram_equalization(image: &DynamicImage) -> RgbaImage {
    // Convert the input image to RGBA format
    let rgba_image = image.to_rgba8();

    // Calculate histogram
    let mut histogram = [0u32; 256];
    for pixel in rgba_image.pixels() {
        let intensity = pixel[0] as usize;
        histogram[intensity] += 1;
    }

    // Calculate cumulative distribution function (CDF)
    let mut cdf = [0u32; 256];
    let mut cumulative_sum = 0;
    for (intensity, &count) in histogram.iter().enumerate() {
        cumulative_sum += count;
        cdf[intensity] = cumulative_sum;
    }

    // Normalize CDF
    let total_pixels = rgba_image.dimensions().0 * rgba_image.dimensions().1;
    let cdf_normalized: Vec<u8> = cdf.iter().map(|&value| ((value as f32 / total_pixels as f32) * 255.0) as u8).collect();

    // Apply histogram equalization
    let mut equalized_image = RgbaImage::new(rgba_image.width(), rgba_image.height());
    for (x, y, pixel) in equalized_image.enumerate_pixels_mut() {
        let intensity = rgba_image.get_pixel(x, y)[0] as usize;
        let equalized_intensity = cdf_normalized[intensity] as u8;
        *pixel = Rgba([equalized_intensity, equalized_intensity, equalized_intensity, 255]);
    }

    equalized_image
}
