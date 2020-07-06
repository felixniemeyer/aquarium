use image::{
    ImageBuffer,
    Rgb, 
    Rgba, 
    Luma, 
    imageops, 
    GenericImageView,
    Pixel, 
    imageops::{
        FilterType,
    },
    DynamicImage,
};

const BG_COLOR: Rgb<u8> = Rgb([20,20,20]);
const COL_DISTANCE_SQUARED: u32 = 45 * 45;

pub fn load_fish_skin(path: String) -> Result<(ImageBuffer<Rgba<u8>, Vec<u8>>, ImageBuffer<Rgb<u8>, Vec<u8>>), String> {
    match image::open(path) {
        Ok(image) => {
            println!("Image color format: {:?}", image.color());
            match image {
                DynamicImage::ImageRgb8(img) => {
                    let dim = img.dimensions(); 
                    let mask = ImageBuffer::from_fn(dim.0, dim.1, |x, y| {
                        if rgb_distance_squared(img.get_pixel(x,y), &BG_COLOR) < COL_DISTANCE_SQUARED {
                            Luma([0 as u16])
                        } else {
                            Luma([std::u16::MAX])
                        }
                    });
                    let mut blurred_mask = imageops::blur(&mask, 3.4); 
                    let mut l = dim.1;
                    let mut r = 0;
                    let mut t = dim.1;
                    let mut b = 0; 
                    for (x, y, pixel) in blurred_mask.enumerate_pixels_mut() {
                        if pixel[0] >= std::u16::MAX - 800 {
                            l = l.min(x);
                            r = r.max(x); 
                            t = t.min(y); 
                            b = b.max(y);  
                            pixel[0] = std::u16::MAX;
                        } else {
                            pixel[0] = 0;
                        }
                    }


                    let center:[u32; 2] = [(l + r) / 2, (t + b) / 2]; 
                    let mut side = (r - l).max(b - t) as i32;
                    side += side % 2; 
                    let square_l = center[0] as i32 - side / 2;
                    let square_t = center[1] as i32 - side / 2;


                    println!("image center: {:?}", center); 
                    println!("side: {:?}", side); 
                    
                    // tried to put it in a function...

                    let square_mask = ImageBuffer::from_fn(side as u32, side as u32, |x, y| {
                        let origx = x as i32 + square_l;
                        let origy = y as i32 + square_t;
                        if origx < 0 || origy < 0 || origx >= dim.0 as i32 || origy >= dim.1 as i32 {
                            Luma([0])
                        } else {
                            blurred_mask.get_pixel(origx as u32, origy as u32).clone()
                        }
                    });
                    
                    let square_img = ImageBuffer::from_fn(side as u32, side as u32, |x, y| {
                        let origx = x as i32 + square_l; 
                        let origy = y as i32 + square_t; 
                        if origx < 0 || origy < 0 || origx >= dim.0 as i32 || origy >= dim.1 as i32 {
                            BG_COLOR
                        } else {
                            img.get_pixel(origx as u32, origy as u32).clone()
                        }
                    });



                    // crop mask to square
                    // - scale to 1024
                    // - scale to 256 x 128
                    //   - blur 
                    //   - unscale
                    

                    let square_mask_1024 = imageops::resize(&square_mask, 1024, 1024, FilterType::CatmullRom);
                    let square_img_1024 = imageops::resize(&square_img, 1024, 1024, FilterType::CatmullRom); 
                    let grey_img_1024 = imageops::colorops::grayscale(&square_img_1024);

                    let w = 128; 
                    let downsampled_mask = imageops::resize(&square_mask, 2*w, 4*w, FilterType::CatmullRom); 
                    let downsampled_mask_with_border = ImageBuffer::from_fn(6*w, 6*w, |x, y| {
                        if x >= 2 * w && x < 4 * w && y >= 1 * w && y < 5 * w {
                            downsampled_mask.get_pixel(x - 2 * w, y - 1 * w).clone()
                        } else {
                            Luma([0])
                        }
                    });
                    let blurred_downsampled_mask = imageops::blur(&downsampled_mask_with_border, w as f32 * 0.25);
                    let cropped_back = ImageBuffer::from_fn(2 * w, 4 * w, |x, y| {
                        blurred_downsampled_mask.get_pixel(x + 2 * w, y + 1 * w).clone()
                    });
                    let heightmap = imageops::resize(&cropped_back, 1024, 1024, FilterType::CatmullRom);

                    let soft_surface = imageops::blur(&grey_img_1024, 2.0); 
                    let heightmap_with_surface = ImageBuffer::from_fn(1024, 1024, |x, y| {
                        let height = heightmap.get_pixel(x,y)[0] as u32; 
                        let surface = soft_surface.get_pixel(x,y)[0] as u32 * (std::u16::MAX as u32 / std::u8::MAX as u32); 
                        Luma([ ( (height * 19 + surface * 1) / 20) as u16 ])
                    }); 

                    let normalmap = ImageBuffer::from_fn(1024, 1024, |x, y| {
                        let pxh =heightmap_with_surface.get_pixel(x,y)[0] as f32 / std::u16::MAX as f32; 
                        let rh = heightmap_with_surface.get_pixel(x.min(1022) + 1,y)[0] as f32 / std::u16::MAX as f32; 
                        let bh = heightmap_with_surface.get_pixel(x,y.min(1022) + 1)[0] as f32 / std::u16::MAX as f32;  
                        // ohne Unterschied ist der Vektor (0,0,-1)
                        let v = [pxh - rh, pxh - bh, 0.005]; 
                        let l = (v[0].powi(2) + v[1].powi(2) + v[2].powi(2)).sqrt();
                        let normalize = |c:f32| {
                            ((c / l + 1.0) * 0.5 * std::u8::MAX as f32) as u8
                        };
                        Rgb([normalize(v[0]),normalize(v[1]),normalize(v[2])])
                    });

                    let colors = ImageBuffer::from_fn(1024, 1024, |x, y| {
                        let color = square_img_1024.get_pixel(x,y); 
                        let alpha = square_mask_1024.get_pixel(x,y); 
                        Rgba([
                             color[0],
                             color[1],
                             color[2],
                             (alpha[0] / (std::u16::MAX as u16 / std::u8::MAX as u16)) as u8
                        ])
                    });

                    // crop colors to square
                    // - greyscale
                    // - add to heightmap

                    // image.crop(
                    //     center[0] - half_side, 
                    //     center[1] - half_side, 
                    //     2 * half_side,
                    //     2 * half_side
                    // );

                    // heightmap to normal map

                    Ok((colors, normalmap))
                },
                _ => panic!("need rgb8 image")
            }
        },
        Err(err) => panic!(err)
    }
}

// pub fn crop_square_and_pad<I: GenericImageView>(
//     orig: &I, 
//     l:i32, 
//     t:i32, 
//     side:u32, 
//     default_color: I::Pixel
//     ) -> ImageBuffer<I::Pixel, Vec< < <I as GenericImageView>::Pixel as Pixel>::Subpixel>> {
//     ImageBuffer::from_fn(side, side, |x, y| {
//         let origx = x as i32 + l;
//         let origy = y as i32 + t;
//         let origdim = orig.dimensions();
//         if origx < 0 || origy < 0 || origx >= origdim.0 as i32 || origy >= origdim.1 as i32 {
//             default_color
//         } else {
//             orig.get_pixel(origx as u32, origy as u32).clone()
//         }
//     })
// }
pub fn rgb_distance_squared(c1: &Rgb<u8>, c2: &Rgb<u8>) -> u32 {
    let mut sum:u32 = 0; 
    for i in 0..2 {
        sum += (c1[i] as i32 - c2[i] as i32).pow(2) as u32
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        //let option = load_fish_skin("./IMG_1539-small.JPG".into());
        let option = load_fish_skin("./IMG_1540.JPG".into());
        assert!(option.is_ok()); 
        let (colors, normals) = option.unwrap(); 
        normals.save("./test_normals.png").unwrap();
        colors.save("./test_colors.png").unwrap();
    }
}


