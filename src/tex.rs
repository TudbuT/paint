use std::sync::Arc;

use egui::*;
use epaint::{ImageDelta, TextureManager};
use micro_ndarray::Array;

use crate::{compress::FlatArea, App};

impl App {
    /// changes the image size when resizing window
    pub fn correct_tex_size(&mut self, texman: &mut TextureManager, window_size: [usize; 2]) {
        if self.image.size() == window_size {
            return;
        }
        // creates a new image and transfers the pixels
        let mut new_image = Array::new_with([window_size[0], window_size[1]], Color32::WHITE);
        for (pos, pixel) in self.image.iter() {
            // if its within the image, set it
            if let Some(px) = new_image.get_mut(pos) {
                *px = *pixel;
            }
        }
        self.image = new_image;

        // everything changed
        self.changes.all(Rect::from_min_max(
            Pos2::ZERO,
            Pos2::new(window_size[0] as f32, window_size[1] as f32),
        ));

        // create a renderable texture from the new image
        let cimg = ColorImage {
            size: self.image.size(),
            pixels: self.image.as_flattened().to_vec(),
        };
        texman.free(self.tex); // drop old texture
        self.tex = texman.alloc(
            "canvas".to_owned(),
            ImageData::Color(Arc::new(cimg)),
            TextureOptions {
                magnification: TextureFilter::Nearest,
                minification: TextureFilter::Linear,
            },
        );
        // write image into the texture
        self.image_to_texture(texman);
    }

    pub fn image_to_texture(&mut self, texman: &mut TextureManager) {
        let changes = self.changes.take();
        // if its so few it can be updated in single pixels
        if let Some(changelist) = changes.changelist {
            for change in changelist {
                let cimg = ColorImage {
                    size: [1, 1],
                    pixels: vec![self.image[change]],
                };
                texman.set(
                    self.tex,
                    ImageDelta::partial(
                        change,
                        cimg,
                        TextureOptions {
                            magnification: TextureFilter::Nearest,
                            minification: TextureFilter::Linear,
                        },
                    ),
                );
            }
            return;
        }
        // if its only a region
        if changes.area < self.image.as_flattened().len() / 2 {
            let cimg = ColorImage {
                size: changes.size,
                pixels: self.image.area_flat(changes.min, changes.size), // only an area of the image, as a flattened array of pixels
            };
            texman.set(
                self.tex,
                ImageDelta::partial(
                    changes.min,
                    cimg,
                    TextureOptions {
                        magnification: TextureFilter::Nearest,
                        minification: TextureFilter::Linear,
                    },
                ),
            );
            return;
        }

        // update all
        let cimg = ColorImage {
            size: self.image.size(),
            pixels: self.image.as_flattened().to_vec(),
        };
        texman.set(
            self.tex,
            ImageDelta::full(
                cimg,
                TextureOptions {
                    magnification: TextureFilter::Nearest,
                    minification: TextureFilter::Linear,
                },
            ),
        );
    }
}
