use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

#[derive(Clone, Copy)]
pub enum Fading {
    Bottom { steps: u8 },
    Top { steps: u8 },
    Left { steps: u8 },
    Right { steps: u8 },
}

pub struct FadedRectangle {
    rect: Rectangle,
    base_color: Rgb888,
    fading: Fading,
}

impl FadedRectangle {
    pub fn new(rect: Rectangle, base_color: Rgb888, fading: Fading) -> Self {
        Self {
            rect,
            base_color,
            fading,
        }
    }

    fn interpolate_color(&self, position: Point) -> Rgb888 {
        let (r, g, b) = (
            self.base_color.r(),
            self.base_color.g(),
            self.base_color.b(),
        );

        let steps = match self.fading {
            Fading::Bottom { steps }
            | Fading::Top { steps }
            | Fading::Left { steps }
            | Fading::Right { steps } => steps as u32,
        };

        let row_in_rect = (position.y - self.rect.top_left.y) as u32;
        let col_in_rect = (position.x - self.rect.top_left.x) as u32;
        let total_height = self.rect.size.height;
        let total_width = self.rect.size.width;

        let fade_factor = match self.fading {
            Fading::Bottom { .. } => {
                if row_in_rect >= total_height.saturating_sub(steps) {
                    let rows_from_start_of_fade = row_in_rect - total_height.saturating_sub(steps);
                    (rows_from_start_of_fade + 1) as f32 / steps as f32
                } else {
                    0.0
                }
            }
            Fading::Top { .. } => {
                if row_in_rect < steps {
                    (steps - row_in_rect) as f32 / steps as f32
                } else {
                    0.0
                }
            }
            Fading::Right { .. } => {
                if col_in_rect >= total_width.saturating_sub(steps) {
                    let cols_from_start_of_fade = col_in_rect - total_width.saturating_sub(steps);
                    (cols_from_start_of_fade + 1) as f32 / steps as f32
                } else {
                    0.0
                }
            }
            Fading::Left { .. } => {
                if col_in_rect < steps {
                    (steps - col_in_rect) as f32 / steps as f32
                } else {
                    0.0
                }
            }
        };

        let new_r = (r as f32 * (1.0 - fade_factor)) as u8;
        let new_g = (g as f32 * (1.0 - fade_factor)) as u8;
        let new_b = (b as f32 * (1.0 - fade_factor)) as u8;

        Rgb888::new(new_r, new_g, new_b)
    }
}

impl Drawable for FadedRectangle {
    type Color = Rgb888;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        target.draw_iter(self)
    }
}

impl IntoIterator for FadedRectangle {
    type IntoIter = FadedRectangleIterator;
    type Item = Pixel<Rgb888>;

    fn into_iter(self) -> Self::IntoIter {
        FadedRectangleIterator {
            rect: self.rect,
            base_color: self.base_color,
            fading: self.fading,
            current_x: self.rect.top_left.x,
            current_y: self.rect.top_left.y,
        }
    }
}

pub struct FadedRectangleIterator {
    rect: Rectangle,
    base_color: Rgb888,
    fading: Fading,
    current_x: i32,
    current_y: i32,
}

impl<'a> IntoIterator for &'a FadedRectangle {
    type IntoIter = FadedRectangleIterator;
    type Item = Pixel<Rgb888>;

    fn into_iter(self) -> Self::IntoIter {
        FadedRectangleIterator {
            rect: self.rect,
            base_color: self.base_color,
            fading: self.fading,
            current_x: self.rect.top_left.x,
            current_y: self.rect.top_left.y,
        }
    }
}

impl Iterator for FadedRectangleIterator {
    type Item = Pixel<Rgb888>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_y >= self.rect.top_left.y + self.rect.size.height as i32 {
            return None;
        }

        let point = Point::new(self.current_x, self.current_y);
        let faded_rect = FadedRectangle {
            rect: self.rect,
            base_color: self.base_color,
            fading: self.fading,
        };
        let color = faded_rect.interpolate_color(point);

        self.current_x += 1;
        if self.current_x >= self.rect.top_left.x + self.rect.size.width as i32 {
            self.current_x = self.rect.top_left.x;
            self.current_y += 1;
        }

        Some(Pixel(point, color))
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

        std::process::Command::new("open")
            .arg(output_path)
            .spawn()
            .ok();
    }
}
