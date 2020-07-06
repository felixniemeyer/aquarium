use image::{
    ImageBuffer,
    Rgb, 
    Luma, 
    DynamicImage::{
        ImageRgb8
    }
};

const BG_COLOR: Rgb<u8> = Rgb([20,20,20]);
const COL_DISTANCE_SQUARED: u32 = 45 * 45;

pub fn load_fish_skin(path: String) -> Result<(ImageBuffer<Rgb<u8>, Vec<u8>>, ImageBuffer<Luma<u8>, Vec<u8>>), String> {
    match image::open(path) {
        Ok(image) => {
            println!("Image color format: {:?}", image.color());
            match image {
                ImageRgb8(img) => {
                    let dim = img.dimensions(); 
                    let mask = ImageBuffer::from_fn(dim.0, dim.1, |x, y| {
                        if rgb_distance_squared(img.get_pixel(x,y), &BG_COLOR) < COL_DISTANCE_SQUARED {
                            Luma([0])
                        } else {
                            Luma([255])
                        }
                    });
                    let mut blurred_mask = image::imageops::blur(&mask, 3.4); 
                    let mut l = dim.1;
                    let mut r = 0;
                    let mut t = dim.1;
                    let mut b = 0; 
                    for (x, y, pixel) in blurred_mask.enumerate_pixels_mut() {
                        if pixel[0] >= 253 {
                            l = l.min(x);
                            r = r.max(x); 
                            t = t.min(y); 
                            b = b.max(y); 
                        } else {
                            pixel[0] = 0;
                        }
                    }

                    let center:[u32; 2] = [(l + r) / 2, (t + b) / 2]; 
                    let half_side = ( (r - l).max(b - t) as f32 * 1.05 ) as u32;

                    // crop mask to square
                    // - scale to 1024
                    // - scale to 256 x 128
                    //   - blur 
                    //   - unscale
                    // crop colors to square
                    // - greyscale
                    // - add to heightmap
                    // heightmap to normal map

                    Ok((img, blurred_mask))
                },
                _ => panic!("need rgb8 image")
            }
        },
        Err(err) => panic!(err)
    }
}

pub fn rgb_distance_squared(c1: &Rgb<u8>, c2: &Rgb<u8>) -> u32 {
    let mut sum:u32 = 0; 
    for i in 0..2 {
        sum += (c1[i] as u32 - c2[i] as u32).pow(2)
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let option = load_fish_skin("./IMG_1539.JPG".into());
        assert!(option.is_ok()); 
        let (colors, normals) = option.unwrap(); 
        normals.save("./test_result_normals.png").unwrap();
    }
}


