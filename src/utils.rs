use error::{Error, Result};
use sfml::graphics::{Color, Image, IntRect, RenderTarget, RenderTexture, Sprite, Texture,
                     Transformable};
use sfml::system::Vector2f;
use unitypack::engine::texture::Texture2D;

pub trait ImageUtils {
    fn remove_transparency(&mut self);
    fn resize(&mut self, width: u32, height: u32) -> Result<Image>;
}

impl ImageUtils for Image {
    fn remove_transparency(&mut self) {
        for x in 0..self.size().x {
            for y in 0..self.size().y {
                let mut c = self.pixel_at(x, y);
                c.a = 255;
                self.set_pixel(x, y, &c);
            }
        }
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<Image> {
        let mut texture = Texture::from_image(&self).ok_or(Error::SFMLError)?;
        texture.set_smooth(true);
        let mut tmp_sprite = Sprite::with_texture(&texture);
        let scale = Vector2f {
            x: width as f32 / tmp_sprite.local_bounds().width,
            y: height as f32 / tmp_sprite.local_bounds().height,
        };

        tmp_sprite.set_scale(scale);

        let mut canvas = RenderTexture::new(width, height, false).ok_or(Error::SFMLError)?;
        canvas.set_smooth(true);
        let transparent_color = Color::rgba(0, 0, 0, 0);
        canvas.clear(&transparent_color);
        canvas.draw(&tmp_sprite);
        canvas.display();

        canvas.texture().copy_to_image().ok_or(Error::SFMLError)
    }
}

pub trait SpriteTransforms {
    fn flip_horizontally(&mut self);
    fn flip_vertically(&mut self);
}

impl<'s> SpriteTransforms for Sprite<'s> {
    fn flip_horizontally(&mut self) {
        let texture_rect = self.texture_rect();
        self.set_texture_rect(&IntRect::new(
            0,
            texture_rect.height,
            texture_rect.width,
            -1 * texture_rect.height,
        ));
    }
    fn flip_vertically(&mut self) {
        let texture_rect = self.texture_rect();
        self.set_texture_rect(&IntRect::new(
            texture_rect.width,
            0,
            -1 * texture_rect.width,
            texture_rect.height,
        ));
    }
}

pub trait TextureExtensions {
    fn save_to_file(&self, filepath: &str) -> Result<()>;
}

impl TextureExtensions for RenderTexture {
    fn save_to_file(&self, filepath: &str) -> Result<()> {
        let texture = self.texture();
        let img = texture.copy_to_image().ok_or(Error::SFMLError)?;
        img.save_to_file(filepath);
        Ok(())
    }
}

pub trait IntoImage {
    fn to_sfml_image(self) -> Result<Image>;
}

impl IntoImage for Texture2D {
    fn to_sfml_image(self) -> Result<Image> {
        Image::create_from_pixels(self.width, self.height, &self.to_image()?)
            .ok_or(Error::SFMLError)
    }
}
