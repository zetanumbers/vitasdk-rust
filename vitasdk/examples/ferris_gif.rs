#![deny(unsafe_code)]

use std::{fs::File, io, iter, time::Duration};

use ab_glyph::{Font, Glyph, ScaleFont};
use image::{imageops, AnimationDecoder};
use vitasdk::display::{self, Display};

const LABEL_VERTICAL: f64 = 0.9;
const GIF_VERTIVAL: f64 = 0.2;

fn main() {
    let mut display = Display::take().unwrap();
    let mut fb = alloc_framebuffer();
    display.replace_framebuf(alloc_framebuffer()).unwrap();
    display.wait_set_framebuf().unwrap();

    {
        let font = &ab_glyph::FontVec::try_from_vec(std::fs::read("font.ttf").unwrap()).unwrap();
        let font = font.as_scaled(ab_glyph::PxScale::from(40.0));

        let label = render_text(&font, "Hello, rustlang!");
        render(&label, &mut fb, LABEL_VERTICAL);
        fb = display.replace_framebuf(fb).unwrap().unwrap();
        render(&label, &mut fb, LABEL_VERTICAL);
    }

    let mut decoded_frames = Vec::new();
    {
        let reader = io::BufReader::new(File::open("ferris.gif").unwrap());
        let decoder = image::codecs::gif::GifDecoder::new(reader).unwrap();
        let mut frames = decoder.into_frames();

        let first_frame = frames.next().unwrap();
        for frame in iter::once(first_frame).chain(frames) {
            let frame = frame.unwrap();
            render(frame.buffer(), &mut fb, GIF_VERTIVAL);
            decoded_frames.push(frame);
            fb = display.replace_framebuf(fb).unwrap().unwrap();
        }
    }

    let delay = decoded_frames.first().unwrap().delay();
    assert!(decoded_frames[1..].iter().all(|f| delay == f.delay()));
    let vcount = ((Duration::from(delay).as_secs_f64() * 60.0).round() as u32).max(1);

    loop {
        for frame in &decoded_frames {
            render(frame.buffer(), &mut fb, GIF_VERTIVAL);
            display.wait_vblank_start_multi(vcount).unwrap();
            fb = display.replace_framebuf(fb).unwrap().unwrap();
        }
    }
}

fn alloc_framebuffer() -> display::Framebuf {
    display::Framebuf::native().unwrap()
}

fn render(image: &image::RgbaImage, fb: &mut display::Framebuf, y: f64) {
    let x = (fb.desc.width as i32 - image.width() as i32) / 2;
    let y = ((fb.desc.height as i32 - image.height() as i32) as f64 * y) as u32;
    imageops::replace(
        &mut image_from_fb(fb).as_view_mut().unwrap(),
        image,
        x.into(),
        y.into(),
    );
}

fn image_from_fb(fb: &mut display::Framebuf) -> image::FlatSamples<&mut [u8]> {
    image::FlatSamples {
        samples: &mut *fb.memblock,
        layout: image::flat::SampleLayout {
            channels: 4,
            channel_stride: 1,
            width: fb.desc.width,
            width_stride: 4,
            height: fb.desc.height,
            height_stride: 4 * fb.desc.pitch as usize,
        },
        color_hint: Some(match fb.desc.pixel_format {
            display::PixelFormat::A8B8G8R8 => image::ColorType::Rgba8,
            _ => unimplemented!(),
        }),
    }
}

fn render_text<F, SF>(font: &SF, text: &str) -> image::RgbaImage
where
    F: ab_glyph::Font,
    SF: ScaleFont<F>,
{
    let glyphs =
        layout_paragraph(font, ab_glyph::point(0.0, 0.0), 9999.0, text).collect::<Vec<_>>();

    // work out the layout size
    let glyphs_height = font.height().ceil() as u32;
    let glyphs_width = {
        let min_x = glyphs.first().unwrap().position.x;
        let last_glyph = glyphs.last().unwrap();
        let max_x = last_glyph.position.x + font.h_advance(last_glyph.id);
        (max_x - min_x).ceil() as u32
    };

    // Create a new rgba image with some padding
    let mut image = image::DynamicImage::new_rgba8(glyphs_width, glyphs_height).to_rgba8();

    // Loop through the glyphs in the text, positing each one on a line
    for glyph in glyphs {
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            // Draw the glyph into the image per-pixel by using the draw closure
            outlined.draw(|x, y, v| {
                let v = (v * 255.0) as u8;
                // Offset the position by the glyph bounding box
                image.put_pixel(
                    x + bounds.min.x as u32,
                    y + bounds.min.y as u32,
                    image::Rgba([v, v, v, 255]),
                );
            });
        }
    }
    image
}

fn layout_paragraph<'a, F, SF>(
    font: SF,
    position: ab_glyph::Point,
    max_width: f32,
    text: &'a str,
) -> impl Iterator<Item = Glyph> + 'a
where
    F: Font + 'a,
    SF: ScaleFont<F> + 'a,
{
    let v_advance = font.height() + font.line_gap();
    let mut caret = position + ab_glyph::point(0.0, font.ascent());
    let mut last_glyph: Option<Glyph> = None;
    text.chars().flat_map(move |c| {
        if c.is_control() {
            if c == '\n' {
                caret = ab_glyph::point(position.x, caret.y + v_advance);
                last_glyph = None;
            }
            return None;
        }
        let mut glyph = font.scaled_glyph(c);
        if let Some(previous) = last_glyph.take() {
            caret.x += font.kern(previous.id, glyph.id);
        }
        glyph.position = caret;

        last_glyph = Some(glyph.clone());
        caret.x += font.h_advance(glyph.id);

        if !c.is_whitespace() && caret.x > position.x + max_width {
            caret = ab_glyph::point(position.x, caret.y + v_advance);
            glyph.position = caret;
            last_glyph = None;
        }

        Some(glyph)
    })
}
