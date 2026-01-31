use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    transform::Transform,
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub enum Fading {
    Bottom { steps: u8 },
    Top { steps: u8 },
    Left { steps: u8 },
    Right { steps: u8 },
}

impl Default for Fading {
    fn default() -> Self {
        Fading::Left { steps: 5 }
    }
}

impl Fading {
    fn steps(&self) -> u8 {
        match self {
            Fading::Bottom { steps } => *steps,
            Fading::Top { steps } => *steps,
            Fading::Left { steps } => *steps,
            Fading::Right { steps } => *steps,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[cfg_attr(feature = "defmt", derive(::defmt::Format))]
pub struct FadedRectangle {
    pub rect: Rectangle,
    pub base_color: Rgb888,
    pub fading: Fading,
}

impl FadedRectangle {
    pub fn new(rect: Rectangle, base_color: Rgb888, fading: Fading) -> Self {
        Self {
            rect,
            base_color,
            fading,
        }
    }

    // This currently just draws diff with respect to left sided shrinking/expanding
    pub fn draw_diff<D>(&self, target: &mut D, previous: &Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Rgb888>,
    {
        if self.rect == *previous {
            return Ok(());
        }

        let x_start_old = previous.top_left.x;
        let x_start_new = self.rect.top_left.x;

        let y_diff = previous.size.height;

        if x_start_new > x_start_old {
            // Left sided shrinking
            let x_diff = x_start_new - x_start_old;

            let rec_diff = Rectangle::new(
                Point {
                    x: x_start_old,
                    y: 0,
                },
                Size {
                    width: x_diff as u32,
                    height: y_diff,
                },
            );

            rec_diff
                .into_styled(PrimitiveStyle::with_fill(Rgb888::BLACK))
                .draw(target)?;

            target.draw_iter(self)?;
        } else {
            // Left sided expanding
            let x_diff = x_start_old - x_start_new + self.fading.steps() as i32;

            let rec_diff = Rectangle::new(
                Point {
                    x: x_start_new,
                    y: 0,
                },
                Size {
                    width: x_diff as u32,
                    height: y_diff,
                },
            );

            let rec_faded = FadedRectangle::new(rec_diff, self.base_color, self.fading);
            rec_faded.draw(target)?;
        }

        Ok(())
    }
}

impl Drawable for FadedRectangle {
    type Color = Rgb888;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        self.rect
            .into_styled(PrimitiveStyle::with_fill(self.base_color))
            .draw(target)?;

        target.draw_iter(self)?;

        Ok(())
    }
}

impl IntoIterator for FadedRectangle {
    type IntoIter = FadedRectangleIterator;
    type Item = Pixel<Rgb888>;

    fn into_iter(self) -> Self::IntoIter {
        let steps = match self.fading {
            Fading::Bottom { steps }
            | Fading::Top { steps }
            | Fading::Left { steps }
            | Fading::Right { steps } => steps,
        };

        FadedRectangleIterator {
            rect: self.rect,
            r: self.base_color.r(),
            g: self.base_color.g(),
            b: self.base_color.b(),
            fading: self.fading,
            steps,
            current_x: self.rect.top_left.x,
            current_y: self.rect.top_left.y,
        }
    }
}

impl Transform for FadedRectangle {
    fn translate(&self, by: Point) -> Self {
        self.rect.translate(by);
        *self
    }

    fn translate_mut(&mut self, by: Point) -> &mut Self {
        self.rect.translate_mut(by);
        &mut *self
    }
}

pub struct FadedRectangleIterator {
    rect: Rectangle,
    r: u8,
    g: u8,
    b: u8,
    fading: Fading,
    steps: u8,
    current_x: i32,
    current_y: i32,
}

impl IntoIterator for &FadedRectangle {
    type IntoIter = FadedRectangleIterator;
    type Item = Pixel<Rgb888>;

    fn into_iter(self) -> Self::IntoIter {
        let steps = match self.fading {
            Fading::Bottom { steps }
            | Fading::Top { steps }
            | Fading::Left { steps }
            | Fading::Right { steps } => steps,
        };

        FadedRectangleIterator {
            rect: self.rect,
            r: self.base_color.r(),
            g: self.base_color.g(),
            b: self.base_color.b(),
            fading: self.fading,
            steps,
            current_x: self.rect.top_left.x,
            current_y: self.rect.top_left.y,
        }
    }
}

impl Iterator for FadedRectangleIterator {
    type Item = Pixel<Rgb888>;

    fn next(&mut self) -> Option<Self::Item> {
        let steps = self.steps as u32;
        let total_height = self.rect.size.height;
        let total_width = self.rect.size.width;

        let (start_row, end_row, start_col, end_col) = match self.fading {
            Fading::Bottom { .. } => (
                total_height.saturating_sub(steps),
                total_height,
                0,
                total_width,
            ),
            Fading::Top { .. } => (0, steps, 0, total_width),
            Fading::Right { .. } => (
                0,
                total_height,
                total_width.saturating_sub(steps),
                total_width,
            ),
            Fading::Left { .. } => (0, total_height, 0, steps),
        };

        // Initialize on first call
        if self.current_y == self.rect.top_left.y && self.current_x == self.rect.top_left.x {
            self.current_y = self.rect.top_left.y + start_row as i32;
            self.current_x = self.rect.top_left.x + start_col as i32;
        }

        // Check if we're done
        let row_in_rect = (self.current_y - self.rect.top_left.y) as u32;
        if row_in_rect >= end_row {
            return None;
        }

        let col_in_rect = (self.current_x - self.rect.top_left.x) as u32;
        let point = Point::new(self.current_x, self.current_y);

        // Calculate fade
        let fade_factor_256 = match self.fading {
            Fading::Bottom { .. } => {
                let rows_from_start = row_in_rect - total_height.saturating_sub(steps);
                ((rows_from_start + 1) * 256 / steps) as u16
            }
            Fading::Top { .. } => ((steps - row_in_rect) * 256 / steps) as u16,
            Fading::Right { .. } => {
                let cols_from_start = col_in_rect - total_width.saturating_sub(steps);
                ((cols_from_start + 1) * 256 / steps) as u16
            }
            Fading::Left { .. } => ((steps - col_in_rect) * 256 / steps) as u16,
        };

        let new_r = ((self.r as u16 * (256 - fade_factor_256)) / 256) as u8;
        let new_g = ((self.g as u16 * (256 - fade_factor_256)) / 256) as u8;
        let new_b = ((self.b as u16 * (256 - fade_factor_256)) / 256) as u8;

        // Advance to next pixel in fade zone
        self.current_x += 1;
        if self.current_x >= self.rect.top_left.x + end_col as i32 {
            self.current_x = self.rect.top_left.x + start_col as i32;
            self.current_y += 1;
        }

        Some(Pixel(point, Rgb888::new(new_r, new_g, new_b)))
    }
}

#[cfg(test)]
mod simulator_tests {
    use super::*;
    use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};
    use embedded_graphics_simulator::SimulatorDisplay;

    #[test]
    fn visual_test_bottom_fade() {
        let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(320, 240));

        let rect = Rectangle::new(Point::new(10, 10), Size::new(300, 5));
        let base_color = Rgb888::new(255, 0, 0);
        FadedRectangle::new(rect, base_color, Fading::Bottom { steps: 4 })
            .draw(&mut display)
            .unwrap();

        let output_path = "visual_test_bottom_fade.png";
        display
            .to_rgb_output_image(&Default::default())
            .save_png(output_path)
            .unwrap();

        // std::process::Command::new("open")
        //     .arg(output_path)
        //     .spawn()
        //     .ok();
    }

    #[test]
    fn visual_test_left_fade() {
        let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(320, 240));

        let rect = Rectangle::new(Point::new(10, 10), Size::new(100, 32));
        let base_color = Rgb888::new(0, 255, 132);
        FadedRectangle::new(rect, base_color, Fading::Left { steps: 5 })
            .draw(&mut display)
            .unwrap();

        let output_path = "visual_test_left_fade.png";
        display
            .to_rgb_output_image(&Default::default())
            .save_png(output_path)
            .unwrap();

        // std::process::Command::new("open")
        //     .arg(output_path)
        //     .spawn()
        //     .ok();
    }

    #[test]
    fn visual_test_right_fade() {
        let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(320, 240));

        let rect = Rectangle::new(Point::new(10, 10), Size::new(100, 32));
        let base_color = Rgb888::new(255, 255, 0);
        FadedRectangle::new(rect, base_color, Fading::Right { steps: 5 })
            .draw(&mut display)
            .unwrap();

        let output_path = "visual_test_right_fade.png";
        display
            .to_rgb_output_image(&Default::default())
            .save_png(output_path)
            .unwrap();

        // std::process::Command::new("open")
        //     .arg(output_path)
        //     .spawn()
        //     .ok();
    }

    #[test]
    fn visual_test_top_fade() {
        let mut display = SimulatorDisplay::<Rgb888>::new(Size::new(320, 240));

        let rect = Rectangle::new(Point::new(10, 10), Size::new(300, 20));
        let base_color = Rgb888::new(255, 49, 73);
        FadedRectangle::new(rect, base_color, Fading::Top { steps: 4 })
            .draw(&mut display)
            .unwrap();

        let output_path = "visual_test_top_fade.png";
        display
            .to_rgb_output_image(&Default::default())
            .save_png(output_path)
            .unwrap();

        // std::process::Command::new("open")
        //     .arg(output_path)
        //     .spawn()
        //     .ok();
    }

    #[test]
    fn test_large_shrink() {
        let mut display = SimulatorDisplay::new(Size::new(96, 32));
        let prev_rect = Rectangle::new(Point::new(0, 0), Size::new(96, 32));
        let prev_faded = FadedRectangle::new(prev_rect, Rgb888::GREEN, Fading::Left { steps: 5 });
        prev_faded.draw(&mut display).unwrap();

        let before_output = display.to_rgb_output_image(&Default::default());
        let before_image = before_output.as_image_buffer();

        let new_rect = Rectangle::new(Point::new(76, 0), Size::new(20, 32));
        let new_faded = FadedRectangle::new(new_rect, Rgb888::GREEN, Fading::Left { steps: 5 });

        new_faded.draw_diff(&mut display, &prev_rect).unwrap();

        let after_output = display.to_rgb_output_image(&Default::default());
        let after_image = after_output.as_image_buffer();

        let combined_width = 96 + 20;
        let combined_height = 32 * 2 + 30;

        let mut combined = image::RgbImage::new(combined_width, combined_height);

        for pixel in combined.pixels_mut() {
            *pixel = image::Rgb([32, 32, 32]);
        }

        for y in 0..32 {
            for x in 0..96 {
                let src_pixel = before_image.get_pixel(x, y);
                combined.put_pixel(x + 10, y + 10, *src_pixel);
            }
        }

        for y in 0..32 {
            for x in 0..96 {
                let src_pixel = after_image.get_pixel(x, y);
                combined.put_pixel(x + 10, y + 32 + 20, *src_pixel);
            }
        }

        let output_path = "test_large_shrink_comparison.png";
        combined.save(output_path).unwrap();

        // std::process::Command::new("open")
        //     .arg(output_path)
        //     .spawn()
        //     .ok();
    }

    #[test]
    fn test_expanding() {
        let mut display = SimulatorDisplay::new(Size::new(96, 32));
        let prev_rect = Rectangle::new(Point::new(20, 0), Size::new(76, 32));
        let prev_faded = FadedRectangle::new(prev_rect, Rgb888::YELLOW, Fading::Left { steps: 5 });
        prev_faded.draw(&mut display).unwrap();

        let before_output = display.to_rgb_output_image(&Default::default());
        let before_image = before_output.as_image_buffer();

        let new_rect = Rectangle::new(Point::new(18, 0), Size::new(78, 32));
        let new_faded = FadedRectangle::new(new_rect, Rgb888::YELLOW, Fading::Left { steps: 5 });

        new_faded.draw_diff(&mut display, &prev_rect).unwrap();

        let after_output = display.to_rgb_output_image(&Default::default());
        let after_image = after_output.as_image_buffer();

        let combined_width = 96 + 20;
        let combined_height = 32 * 2 + 30;

        let mut combined = image::RgbImage::new(combined_width, combined_height);

        for pixel in combined.pixels_mut() {
            *pixel = image::Rgb([32, 32, 32]);
        }

        for y in 0..32 {
            for x in 0..96 {
                let src_pixel = before_image.get_pixel(x, y);
                combined.put_pixel(x + 10, y + 10, *src_pixel);
            }
        }

        for y in 0..32 {
            for x in 0..96 {
                let src_pixel = after_image.get_pixel(x, y);
                combined.put_pixel(x + 10, y + 32 + 20, *src_pixel);
            }
        }

        let output_path = "test_expanding_comparison.png";
        combined.save(output_path).unwrap();

        // std::process::Command::new("open")
        //     .arg(output_path)
        //     .spawn()
        //     .ok();
    }
}
