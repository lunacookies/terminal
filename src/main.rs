use crossfont::{
    BitmapBuffer, FontDesc, GlyphKey, Rasterize, RasterizedGlyph, Rasterizer, Size, Style,
};
use pixels::{Error, Pixels, SurfaceTexture};
use std::iter;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

const WIDTH: u32 = 2000;
const HEIGHT: u32 = 1000;
const SIZE: f32 = 13.0;
const TEXT: &str =
    "the quick brown fox jumped over the lazy dog and THE QUICK BROWN FOX JUMPED OVER THE LAZY DOG";

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(PhysicalSize {
            width: WIDTH,
            height: HEIGHT,
        })
        .build(&event_loop)
        .unwrap();

    let scale_factor = window.scale_factor();

    let surface_texture = SurfaceTexture::new(WIDTH, HEIGHT, &window);
    let mut pixels = Pixels::new(WIDTH, HEIGHT, surface_texture)?;
    let mut frame_n = 0;

    let mut rasterizer = Rasterizer::new(scale_factor as f32, false).unwrap();

    let font_key = rasterizer
        .load_font(
            &FontDesc::new("Input Sans", Style::Specific("Light".to_string())),
            Size::new(SIZE),
        )
        .unwrap();

    let width_of_space = calc_with_of_space(&mut rasterizer, font_key);
    let mut y = 100;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                frame_n += 1;
                println!("redraw {}", frame_n);

                let mut screen_pixel_buf = PixelBuf {
                    pixels: vec![Pixel::default(); (WIDTH * HEIGHT) as usize],
                    width: WIDTH as usize,
                };

                let mut x = 100;

                for c in TEXT.chars() {
                    if c == ' ' {
                        x += width_of_space;
                    } else {
                        render_character(
                            c,
                            &mut rasterizer,
                            font_key,
                            &mut screen_pixel_buf,
                            Coordinate { x: 100, y },
                            &mut x,
                        );
                    }
                }

                for (pixel_mut_ref, pixel_value) in pixels
                    .get_frame()
                    .iter_mut()
                    .zip(screen_pixel_buf.subpixels())
                {
                    *pixel_mut_ref = pixel_value;
                }

                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }

                println!("end redraw {}", frame_n);
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { .. } => window.request_redraw(),
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::PixelDelta(PhysicalPosition { y: y_change, .. }),
                    ..
                } => {
                    if y_change > 0.0 {
                        y += y_change as usize;
                    } else {
                        y -= -y_change as usize;
                    }
                    window.request_redraw();
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }
    });
}

fn render_character(
    character: char,
    rasterizer: &mut Rasterizer,
    font_key: crossfont::FontKey,
    screen_pixel_buf: &mut PixelBuf,
    character_pos: Coordinate,
    x: &mut usize,
) {
    let glyph = rasterizer
        .get_glyph(GlyphKey {
            character,
            font_key,
            size: Size::new(SIZE),
        })
        .unwrap();

    let width = glyph.width as usize;
    let left = glyph.left as usize;
    let top = glyph.top as usize;

    let glyph_pixel_buf = PixelBuf::from(glyph);

    for (pixel, coordinate) in glyph_pixel_buf.pixels() {
        screen_pixel_buf.set_pixel(
            Coordinate {
                x: *x + coordinate.x + character_pos.x + left,
                y: coordinate.y + character_pos.y - top,
            },
            pixel,
        );
    }

    *x += width + left;
}

fn calc_with_of_space(rasterizer: &mut Rasterizer, font_key: crossfont::FontKey) -> usize {
    // The letter ‘i’ is usually the same with as a space.
    let glyph = rasterizer
        .get_glyph(GlyphKey {
            character: 'i',
            font_key,
            size: Size::new(SIZE),
        })
        .unwrap();

    (glyph.width + glyph.left) as usize
}

struct PixelBuf {
    pixels: Vec<Pixel>,
    width: usize,
}

impl PixelBuf {
    fn set_pixel(&mut self, coordinate: Coordinate, new_pixel: Pixel) {
        assert!(coordinate.x < self.width);
        self.pixels[coordinate.to_idx(self.width)] = new_pixel;
    }

    fn subpixels(self) -> impl Iterator<Item = u8> {
        self.pixels.into_iter().flat_map(Pixel::iter)
    }

    fn pixels(self) -> impl Iterator<Item = (Pixel, Coordinate)> {
        let width = self.width;

        self.pixels
            .into_iter()
            .enumerate()
            .map(move |(idx, pixel)| {
                (
                    pixel,
                    Coordinate {
                        x: idx % width,
                        y: idx / width,
                    },
                )
            })
    }
}

impl From<RasterizedGlyph> for PixelBuf {
    fn from(glyph: RasterizedGlyph) -> Self {
        let width = glyph.width as usize;

        match glyph.buffer {
            BitmapBuffer::RGB(rgb) => PixelBuf {
                pixels: rgb
                    .chunks(3)
                    .map(|pixel| Pixel {
                        r: pixel[0],
                        g: pixel[1],
                        b: pixel[2],
                        a: 255,
                    })
                    .collect(),
                width,
            },

            BitmapBuffer::RGBA(rgba) => PixelBuf {
                pixels: rgba
                    .chunks(4)
                    .map(|pixel| Pixel {
                        r: pixel[0],
                        g: pixel[1],
                        b: pixel[2],
                        a: pixel[3],
                    })
                    .collect(),
                width,
            },
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Pixel {
    fn iter(self) -> impl Iterator<Item = u8> {
        iter::once(self.r)
            .chain(iter::once(self.g))
            .chain(iter::once(self.b))
            .chain(iter::once(self.a))
    }
}

#[derive(Clone, Copy)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl Coordinate {
    fn to_idx(self, width: usize) -> usize {
        self.y * width + self.x
    }
}
