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
    let mut y = 100;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(_) => {
                redraw(
                    &mut frame_n,
                    width_of_space,
                    &mut rasterizer,
                    font_key,
                    &mut y,
                    &mut pixels,
                    control_flow,
                );
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::MouseInput { .. } => window.request_redraw(),
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::PixelDelta(PhysicalPosition { y: y_change, .. }),
                    ..
                } => {
                    y += y_change as isize;
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
    y: &mut isize,
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
        y,
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
    screen_pixel_buf: &mut PixelBuf,
    y: &mut isize,
) {
    let mut x = 100;

    for c in text.chars() {
        if c == ' ' {
            x += width_of_space as isize;
        } else {
            render_character(
                c,
                rasterizer,
                font_key,
                screen_pixel_buf,
                Coordinate { x: 100, y: *y },
                &mut x,
            );
        }
    }
}

fn gen_screen_pixel_buf() -> PixelBuf {
    PixelBuf {
        pixels: vec![Pixel::default(); (WIDTH * HEIGHT) as usize],
        width: WIDTH as usize,
        height: HEIGHT as usize,
    }
}

fn render_character(
    character: char,
    rasterizer: &mut Rasterizer,
    font_key: crossfont::FontKey,
    screen_pixel_buf: &mut PixelBuf,
    character_pos: Coordinate,
    x: &mut isize,
) {
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
                x: *x + coordinate.x + character_pos.x + left,
                y: coordinate.y + character_pos.y - top,
            },
            pixel,
        );
    }

    *x += (width + left) as isize;
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
    height: usize,
}

impl PixelBuf {
    fn set_pixel(&mut self, coordinate: Coordinate, new_pixel: Pixel) {
        if let Some(idx) = coordinate.to_idx(self.width, self.height) {
            self.pixels[idx] = new_pixel;
        }
    }

    fn write_to_rgba_buffer(self, rgba: &mut [u8]) {
        let mut idx = 0;

        for Pixel { r, g, b, a } in self.pixels {
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = a;

            idx += 4;
        }
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
                        x: (idx % width) as isize,
                        y: (idx / width) as isize,
                    },
                )
            })
    }
}

impl From<RasterizedGlyph> for PixelBuf {
    fn from(glyph: RasterizedGlyph) -> Self {
        let width = glyph.width as usize;
        let height = glyph.height as usize;

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
                height,
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
                height,
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
