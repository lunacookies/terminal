use crossfont::{
    BitmapBuffer, FontDesc, GlyphKey, Rasterize, RasterizedGlyph, Rasterizer, Size, Style,
};
use pixels::{Error, Pixels, SurfaceTexture};
use std::iter;
use winit::dpi::PhysicalSize;
use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 500;

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
            Size::new(13.0),
        )
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                frame_n += 1;
                println!("redraw {}", frame_n);

                let glyph = rasterizer
                    .get_glyph(GlyphKey {
                        character: 'g',
                        font_key,
                        size: Size::new(13.0),
                    })
                    .unwrap();

                let glyph_pixel_buf = PixelBuf::from(glyph);

                let mut screen_pixel_buf = PixelBuf {
                    pixels: vec![Pixel::default(); (WIDTH * HEIGHT) as usize],
                    width: WIDTH as usize,
                };

                for (pixel, coordinate) in glyph_pixel_buf.pixels() {
                    screen_pixel_buf.set_pixel(
                        Coordinate {
                            x: coordinate.x + 100,
                            y: coordinate.y + 100,
                        },
                        pixel,
                    );
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
            }
            Event::WindowEvent { event, .. } => match event {
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

struct PixelBuf {
    pixels: Vec<Pixel>,
    width: usize,
}

impl PixelBuf {
    fn set_pixel(&mut self, coordinate: Coordinate, new_pixel: Pixel) {
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
