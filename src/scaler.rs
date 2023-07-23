use std::{borrow::Cow, ops::Range};

use arboard::ImageData;

use crate::AsciiImage;

pub fn scale<'a>(image_data: &'a ImageData, end_width: usize, end_height: usize) -> AsciiImage<'a> {
    let mut grayscale_data: Vec<u8> = Vec::new();

    for chunk in image_data.bytes.chunks_exact(4) {
        grayscale_data.push(pixel_to_grayscale(chunk));
    }

    let horizontal_ratio: f32 = image_data.width as f32 / end_width as f32;
    let vertical_ratio: f32 = image_data.height as f32 / end_height as f32;
    let mut output_data: Vec<u8> = Vec::new();

    let horizontal_ranges = get_ranges_by_ratio(end_width, horizontal_ratio);
    let vertical_ranges = get_ranges_by_ratio(end_height, vertical_ratio);

    for y_range in vertical_ranges.into_iter() {
        for x_range in horizontal_ranges.clone().into_iter() {
            let mut pixels_to_average: Vec<u8> = Vec::new();

            for i in y_range.clone() {
                let row_index_padding = i * image_data.width;

                for j in x_range.clone() {
                    match grayscale_data.get(row_index_padding + j) {
                        Some(&pixel) => pixels_to_average.push(pixel),
                        None => panic!("Tried accessing outisde of image bounds"),
                    }
                }
            }

            if pixels_to_average.is_empty() {
                output_data.push(0);
                continue;
            }

            let pixel_count = pixels_to_average.len() as u32;
            let sum: u32 = pixels_to_average
                .into_iter()
                .fold(0, |accumulator, e| accumulator + e as u32);
            output_data.push((sum / pixel_count) as u8);
        }
    }

    return AsciiImage {
        width: end_width,
        height: end_height,
        data: Cow::from(output_data),
    };
}

fn pixel_to_grayscale(chunk: &[u8]) -> u8 {
    let red = chunk[0] as u32;
    let green = chunk[1] as u32;
    let blue = chunk[2] as u32;
    let alpha = chunk[3] as u32;

    // average color channels
    let average = (red + green + blue) / 3;

    // cap value to alpha
    return average.min(alpha) as u8;
}

fn get_ranges_by_ratio(size: usize, ratio: f32) -> Vec<Range<usize>> {
    let mut counter: f32 = 0.0;
    let mut previous_counter_int: usize;

    let mut ranges: Vec<Range<usize>> = Vec::new();
    for _ in 0..size {
        previous_counter_int = counter.trunc() as usize;
        counter += ratio;
        ranges.push(previous_counter_int..counter.trunc() as usize);
    }

    return ranges;
}

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use arboard::ImageData;

    use crate::scaler::get_ranges_by_ratio;

    use super::scale;

    #[test]
    fn get_ranges_same() {
        let expected_ranges = Vec::from([0..1, 1..2, 2..3, 3..4]);
        assert_eq!(expected_ranges, get_ranges_by_ratio(4, 1.0));
    }

    #[test]
    fn get_ranges_increase() {
        let expected_ranges = Vec::from([
            0..2,
            2..5,
            5..7,
            7..10,
            10..12,
            12..15,
            15..17,
            17..20,
            20..22,
            22..25,
        ]);
        assert_eq!(expected_ranges, get_ranges_by_ratio(10, 2.5));
    }

    #[test]
    fn get_ranges_decrease() {
        let expected_ranges = Vec::from([0..1, 1..3, 3..4]);
        assert_eq!(expected_ranges, get_ranges_by_ratio(3, 1.5));
    }

    #[test]
    fn empty_case() {
        let image = ImageData {
            width: 0,
            height: 0,
            bytes: Cow::from(Vec::new()),
        };
        let scaled = scale(&image, 0, 0);
        assert_eq!(0, scaled.width);
        assert_eq!(0, scaled.height);
        assert_eq!(0, scaled.data.len());
    }

    #[test]
    fn same_size() {
        let bytes: Vec<u8> = Vec::from([5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5]);
        let image = ImageData {
            width: 2,
            height: 2,
            bytes: Cow::from(bytes),
        };
        let scaled = scale(&image, 2, 2);
        assert_eq!(2, scaled.width);
        assert_eq!(2, scaled.height);
        assert_eq!(4, scaled.data.len());
    }

    #[test]
    fn same_size_averaged() {
        let bytes: Vec<u8> = Vec::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let image = ImageData {
            width: 2,
            height: 2,
            bytes: Cow::from(bytes),
        };
        let expected_bytes: Vec<u8> = Vec::from([1, 5, 9, 13]);
        let scaled = scale(&image, 2, 2);
        assert_eq!(2, scaled.width);
        assert_eq!(2, scaled.height);
        assert_eq!(expected_bytes, scaled.data.into_owned());
    }

    #[test]
    fn scaled_up_average() {
        let bytes: Vec<u8> = Vec::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
        let image = ImageData {
            width: 2,
            height: 2,
            bytes: Cow::from(bytes),
        };
        let expected_bytes: Vec<u8> = Vec::from([0, 0, 0, 0, 1, 5, 0, 9, 13]);
        let scaled = scale(&image, 3, 3);
        assert_eq!(3, scaled.width);
        assert_eq!(3, scaled.height);
        assert_eq!(expected_bytes, scaled.data.into_owned());
    }

    #[test]
    fn scaled_down_average() {
        let bytes: Vec<u8> = Vec::from([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
            47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64,
        ]);
        let image = ImageData {
            width: 4,
            height: 4,
            bytes: Cow::from(bytes),
        };
        let expected_bytes: Vec<u8> = Vec::from([1, 5, 11, 17, 21, 27, 42, 46, 52]);
        let scaled = scale(&image, 3, 3);
        assert_eq!(3, scaled.width);
        assert_eq!(3, scaled.height);
        assert_eq!(expected_bytes, scaled.data.into_owned());
    }
}
