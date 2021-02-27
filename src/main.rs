use crossfont::{
    BitmapBuffer, FontDesc, FontKey, GlyphKey, Rasterize, RasterizedGlyph, Rasterizer, Size, Style,
};
use pixels::{Error, Pixels, SurfaceTexture};
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
    let mut pos = Coordinate { x: 100, y: 100 };

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                redraw(
                    &mut frame_n,
                    width_of_space,
                    &mut rasterizer,
                    font_key,
                    pos,
                    &mut pixels,
                    control_flow,
                );
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { .. } => window.request_redraw(),
                WindowEvent::MouseWheel {
                    delta:
                        MouseScrollDelta::PixelDelta(PhysicalPosition {
                            x: x_change,
                            y: y_change,
                        }),
                    ..
                } => {
                    pos.x -= x_change as isize;
                    pos.y += y_change as isize;
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

fn redraw(
    frame_n: &mut i32,
    width_of_space: usize,
    rasterizer: &mut Rasterizer,
    font_key: FontKey,
    pos: Coordinate,
    pixels: &mut Pixels<winit::window::Window>,
    control_flow: &mut ControlFlow,
) {
    *frame_n += 1;
    println!("redraw {}", frame_n);

    let mut screen_pixel_buf = gen_screen_pixel_buf();

    render_text(
        TEXT,
        width_of_space,
        rasterizer,
        font_key,
        &mut screen_pixel_buf,
        pos,
    );

    screen_pixel_buf.write_to_rgba_buffer(&mut pixels.get_frame());

    if pixels.render().is_err() {
        *control_flow = ControlFlow::Exit;
    }

    println!("end redraw {}", frame_n);
}

fn render_text(
    text: &str,
    width_of_space: usize,
    rasterizer: &mut Rasterizer,
    font_key: FontKey,
    screen_pixel_buf: &mut PixelBuf<Rgb>,
    mut pos: Coordinate,
) {
    for c in text.chars() {
        if c == ' ' {
            pos.x += width_of_space as isize;
        } else {
            pos.x += render_character(c, rasterizer, font_key, screen_pixel_buf, pos);
        }
    }
}

fn gen_screen_pixel_buf() -> PixelBuf<Rgb> {
    PixelBuf {
        pixels: vec![Rgb::default(); (WIDTH * HEIGHT) as usize],
        width: WIDTH as usize,
        height: HEIGHT as usize,
    }
}

fn render_character(
    character: char,
    rasterizer: &mut Rasterizer,
    font_key: crossfont::FontKey,
    screen_pixel_buf: &mut PixelBuf<Rgb>,
    character_pos: Coordinate,
) -> isize {
    let glyph = rasterizer
        .get_glyph(GlyphKey {
            character,
            font_key,
            size: Size::new(SIZE),
        })
        .unwrap();

    let width = glyph.width as isize;
    let left = glyph.left as isize;
    let top = glyph.top as isize;

    let glyph_pixel_buf = PixelBuf::from(glyph);

    for (pixel, coordinate) in glyph_pixel_buf.pixels() {
        screen_pixel_buf.set_pixel(
            Coordinate {
                x: coordinate.x + character_pos.x + left,
                y: coordinate.y + character_pos.y - top,
            },
            Rgba {
                r: 255,
                g: 255,
                b: 255,
                a: pixel.0,
            },
        );
    }

    width + left
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

struct PixelBuf<P> {
    pixels: Vec<P>,
    width: usize,
    height: usize,
}

impl<P> PixelBuf<P> {
    fn pixels(self) -> impl Iterator<Item = (P, Coordinate)> {
        let width = self.width;

        self.pixels
            .into_iter()
            .enumerate()
            .map(move |(idx, pixel)| {
                (
                    pixel,
                    Coordinate {
                        x: (idx % width) as isize,
                        y: (idx / width) as isize,
                    },
                )
            })
    }
}

impl PixelBuf<Rgb> {
    fn set_pixel(&mut self, coordinate: Coordinate, new_pixel: Rgba) {
        if let Some(idx) = coordinate.to_idx(self.width, self.height) {
            self.pixels[idx] = self.pixels[idx].clone().blend(new_pixel);
        }
    }

    fn write_to_rgba_buffer(self, rgba: &mut [u8]) {
        let mut idx = 0;

        for Rgb { r, g, b } in self.pixels {
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = 255;

            idx += 4;
        }
    }
}

impl From<RasterizedGlyph> for PixelBuf<Luma> {
    fn from(glyph: RasterizedGlyph) -> Self {
        let width = glyph.width as usize;
        let height = glyph.height as usize;

        let lumas = match glyph.buffer {
            BitmapBuffer::RGB(rgb) => rgb.into_iter().step_by(3),
            BitmapBuffer::RGBA(rgba) => rgba.into_iter().step_by(4),
        };

        PixelBuf {
            pixels: lumas.map(Luma).collect(),
            width,
            height,
        }
    }
}

#[derive(Clone, Default)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    fn blend(self, other: Rgba) -> Self {
        let bottom = ultraviolet::Vec3 {
            x: self.r as f32,
            y: self.g as f32,
            z: self.b as f32,
        };

        let top = ultraviolet::Vec3 {
            x: other.r as f32,
            y: other.g as f32,
            z: other.b as f32,
        };

        let alpha = other.a as f32 / 255.0;
        let blended = bottom * (1.0 - alpha) + top * alpha;

        Self {
            r: blended.x.round() as u8,
            g: blended.y.round() as u8,
            b: blended.z.round() as u8,
        }
    }
}

struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct Luma(u8);

#[derive(Clone, Copy)]
struct Coordinate {
    x: isize,
    y: isize,
}

impl Coordinate {
    fn to_idx(self, width: usize, height: usize) -> Option<usize> {
        if !(0..width as isize).contains(&self.x) || !(0..height as isize).contains(&self.y) {
            return None;
        }

        Some(self.y as usize * width + self.x as usize)
    }
}
