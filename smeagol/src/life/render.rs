use crate::Life;
use png::HasParameters;

/// Methods to render a Life board.
impl Life {
    #[cfg(feature = "export-png")]
    pub fn save_png<P>(&mut self, path: P)
    where
        P: AsRef<std::path::Path>,
    {
        let file = std::fs::File::create(path).unwrap();
        let writer = std::io::BufWriter::new(file);

        let alive_cells = self.root.get_alive_cells(&mut self.store);
        if !alive_cells.is_empty() {
            let x_min = alive_cells.iter().map(|(x, _)| x).min().cloned().unwrap();
            let y_min = alive_cells.iter().map(|(_, y)| y).min().cloned().unwrap();
            let x_max = alive_cells.iter().map(|(x, _)| x).max().cloned().unwrap();
            let y_max = alive_cells.iter().map(|(_, y)| y).max().cloned().unwrap();
            let width = x_max - x_min + 1;
            let height = y_max - y_min + 1;

            // white rectangle
            let mut data = vec![255u8; (width * height) as usize];
            for &(x, y) in &alive_cells {
                let offset_x = x - x_min;
                let offset_y = y - y_min;
                let index = (offset_y * width + offset_x) as usize;
                data[index] = 0;
            }

            let mut encoder = png::Encoder::new(writer, width as u32, height as u32);
            encoder
                .set(png::ColorType::Grayscale)
                .set(png::BitDepth::Eight);

            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&data).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_png() {
        let mut life = Life::from_rle_pattern(b"bob$2bo$3o!").unwrap();
        let temp_dir = tempfile::tempdir().unwrap();
        life.save_png(temp_dir.path().join("glider.rle"));
    }
}
