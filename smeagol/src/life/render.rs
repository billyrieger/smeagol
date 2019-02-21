use png::HasParameters;
use crate::{BoundingBox, Life};

impl Life {
    pub fn save_png<P>(
        &mut self,
        path: P,
        bounding_box: BoundingBox,
        zoom: u8,
    ) -> std::io::Result<()>
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);

        let zoom_factor = 1 << zoom;
        let zoom_factor_minus_one = zoom_factor - 1;

        let width = (((bounding_box.lower_right.x - bounding_box.upper_left.x) as f64)
            / (zoom_factor as f64))
            .ceil() as i64;
        let height = (((bounding_box.lower_right.y - bounding_box.upper_left.y) as f64)
            / (zoom_factor as f64))
            .ceil() as i64;

        // white rectangle
        let mut data = vec![255u8; (width * height) as usize];

        for img_y in 0..height {
            for img_x in 0..width {
                let bbox = BoundingBox::new(
                    bounding_box
                        .upper_left
                        .offset(img_x * zoom_factor, img_y * zoom_factor),
                    bounding_box
                        .upper_left
                        .offset(img_x * zoom_factor + zoom_factor_minus_one, img_y * zoom_factor + zoom_factor_minus_one),
                );
                if self.contains_alive_cells(bbox) {
                    data[(img_y * width + img_x) as usize] = 0;
                }
            }
        }

        let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
        encoder
            .set(png::ColorType::Grayscale)
            .set(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        let mut life = Life::from_rle_file("../assets/breeder1.rle").unwrap();
        life.set_step_log_2(10);
        life.step();
        life.save_png(std::env::temp_dir().join("out.png"), life.bounding_box().unwrap().pad(10), 0).unwrap();
    }
}
