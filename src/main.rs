mod chip8;

use chip8::Core;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::time::Duration;

// Keybinds:
// 1 2 3 4
// Q W E R
// A S D F
// Z X C V

const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;

fn render_display(display: &[u8; 64 * 32], canvas: &mut Canvas<Window>) {
    let (win_width, win_height) = canvas.window().drawable_size();

    let pixel_width = (win_width as f32 / DISPLAY_WIDTH as f32).round() as u32;
    let pixel_height = (win_height as f32 / DISPLAY_HEIGHT as f32).round() as u32;

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    canvas.set_draw_color(Color::GREEN);
    for y in 0..DISPLAY_HEIGHT {
        for x in 0..DISPLAY_WIDTH {
            let index = y * DISPLAY_WIDTH + x;
            if display[index] != 0 {
                let rect = Rect::new(
                    (x * pixel_width as usize) as i32,
                    (y * pixel_height as usize) as i32,
                    pixel_width,
                    pixel_height,
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
        .window("CHIP8", 640, 320)
        .position_centered()
        .resizable()
        .fullscreen_desktop()
        .build()
        .unwrap();

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let rom_file_path = current_dir.to_str().unwrap_or("");

    let mut emulator = Core::new(rom_file_path);
    let processor = &mut emulator.processor;

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
                    emulator.processor.debug_print();
                    break 'running;
                }

                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => processor.keypad[0x1] = true,
                    Keycode::Num2 => processor.keypad[0x2] = true,
                    Keycode::Num3 => processor.keypad[0x3] = true,
                    Keycode::Num4 => processor.keypad[0xC] = true,
                    Keycode::Q => processor.keypad[0x4] = true,
                    Keycode::W => processor.keypad[0x5] = true,
                    Keycode::E => processor.keypad[0x6] = true,
                    Keycode::R => processor.keypad[0xD] = true,
                    Keycode::A => processor.keypad[0x7] = true,
                    Keycode::S => processor.keypad[0x8] = true,
                    Keycode::D => processor.keypad[0x9] = true,
                    Keycode::F => processor.keypad[0xE] = true,
                    Keycode::Z => processor.keypad[0xA] = true,
                    Keycode::X => processor.keypad[0x0] = true,
                    Keycode::C => processor.keypad[0xB] = true,
                    Keycode::V => processor.keypad[0xF] = true,
                    _ => {}
                },

                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Num1 => processor.keypad[0x1] = false,
                    Keycode::Num2 => processor.keypad[0x2] = false,
                    Keycode::Num3 => processor.keypad[0x3] = false,
                    Keycode::Num4 => processor.keypad[0xC] = false,
                    Keycode::Q => processor.keypad[0x4] = false,
                    Keycode::W => processor.keypad[0x5] = false,
                    Keycode::E => processor.keypad[0x6] = false,
                    Keycode::R => processor.keypad[0xD] = false,
                    Keycode::A => processor.keypad[0x7] = false,
                    Keycode::S => processor.keypad[0x8] = false,
                    Keycode::D => processor.keypad[0x9] = false,
                    Keycode::F => processor.keypad[0xE] = false,
                    Keycode::Z => processor.keypad[0xA] = false,
                    Keycode::X => processor.keypad[0x0] = false,
                    Keycode::C => processor.keypad[0xB] = false,
                    Keycode::V => processor.keypad[0xF] = false,
                    _ => {}
                },
                _ => {}
            }
        }

        processor.cycle();
        render_display(&processor.display, &mut canvas);
        std::thread::sleep(Duration::from_micros(200));
    }

    Ok(())
}
