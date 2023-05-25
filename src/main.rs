mod processor;

use crate::processor::{Processor, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;

// Keybinds:
// 1 2 3 4
// Q W E R
// A S D F
// Z X C V

fn render_display(display: &[u8; 64 * 32], canvas: &mut Canvas<Window>) {
    let (win_width, win_height) = canvas.window().drawable_size();

    let pixel_width = (win_width as f32 / DISPLAY_WIDTH as f32).round() as u16;
    let pixel_height = (win_height as f32 / DISPLAY_HEIGHT as f32).round() as u16;

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    canvas.set_draw_color(Color::GREEN);
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            let index = y * DISPLAY_WIDTH + x;
            if display[index as usize] != 0 {
                let rect = Rect::new(
                    (x * pixel_width) as i32,
                    (y * pixel_height) as i32,
                    pixel_width as u32,
                    pixel_height as u32,
                );
                canvas.fill_rect(rect).unwrap();
            }
        }
    }

    canvas.present();
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut current_dir = env::current_exe().unwrap();

    if args.len() > 2 {
        eprintln!("Usage: {} <ROM file path>", args[0]);
        std::process::exit(1);
    }

    current_dir.pop();
    current_dir.push(&args[1]);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("CHIP8", 0, 0)
        .position_centered()
        .fullscreen_desktop()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let rom_file_path = current_dir.to_str().unwrap_or("");

    let mut processor = Processor::new(rom_file_path);
    processor.debug_print();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    println!("PROCESSOR STATE AT ROM EXIT:");
                    processor.debug_print();
                    break 'running;
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => processor.keypad |= 1,
                    Keycode::Num2 => processor.keypad |= 2,
                    Keycode::Num3 => processor.keypad |= 1 << 2,
                    Keycode::Num4 => processor.keypad |= 1 << 3,
                    Keycode::Q => processor.keypad |= 1 << 4,
                    Keycode::W => processor.keypad |= 1 << 5,
                    Keycode::E => processor.keypad |= 1 << 6,
                    Keycode::R => processor.keypad |= 1 << 7,
                    Keycode::A => processor.keypad |= 1 << 8,
                    Keycode::S => processor.keypad |= 1 << 9,
                    Keycode::D => processor.keypad |= 1 << 10,
                    Keycode::F => processor.keypad |= 1 << 11,
                    Keycode::Z => processor.keypad |= 1 << 12,
                    Keycode::X => processor.keypad |= 1 << 13,
                    Keycode::C => processor.keypad |= 1 << 14,
                    Keycode::V => processor.keypad |= 1 << 15,
                    _ => {}
                },

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => processor.keypad &= !1,
                    Keycode::Num2 => processor.keypad &= !2,
                    Keycode::Num3 => processor.keypad &= !(1 << 2),
                    Keycode::Num4 => processor.keypad &= !(1 << 3),
                    Keycode::Q => processor.keypad &= !(1 << 4),
                    Keycode::W => processor.keypad &= !(1 << 5),
                    Keycode::E => processor.keypad &= !(1 << 6),
                    Keycode::R => processor.keypad &= !(1 << 7),
                    Keycode::A => processor.keypad &= !(1 << 8),
                    Keycode::S => processor.keypad &= !(1 << 9),
                    Keycode::D => processor.keypad &= !(1 << 10),
                    Keycode::F => processor.keypad &= !(1 << 11),
                    Keycode::Z => processor.keypad &= !(1 << 12),
                    Keycode::X => processor.keypad &= !(1 << 13),
                    Keycode::C => processor.keypad &= !(1 << 14),
                    Keycode::V => processor.keypad &= !(1 << 15),
                    _ => {}
                },
                _ => {}
            }
        }

        processor.cycle();
        render_display(&processor.display, &mut canvas);
        std::thread::sleep(std::time::Duration::from_micros(200));
    }

    Ok(())
}
