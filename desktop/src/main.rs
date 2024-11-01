use chip8_core::*;

use std::env;
use std::fs::File;
use std::io::Read;
use std::thread::sleep;
use std::time::{Duration, Instant};

use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{self, Color};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

const MCYCLE: usize = 60;
// Chip8 spec does not mention who quickly the system should actually run.
// In general, 10 is a nice sweet spot.
const TICKS_PER_FRAME: usize = 10; // 600Hz if M-Cycle is 60Hz

fn draw_screen(emu: &Emu, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();
    // Now set draw color to white, iterate through each point and see if it should be drawn
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            // Convert our 1D array's index into a 2D (x, y) position
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_WIDTH) as u32;

            // Draw a rectangle at (x, y), scaled up by our SCALE value
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
    canvas.present();
}

fn key2btn(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

// https://github.com/Rust-SDL2/rust-sdl2/blob/master/examples/audio-queue-squarewave.rs
fn play_audio(emu: &Emu, audio_queue: &mut AudioQueue<i16>) {
    if *emu.get_st() > 0 {
        // Generate a square wave
        let tone_volume = 1_000i16;
        let period = 48_000 / 256;
        let sample_count = 48_000 * 2; // 1s
        let mut wav = Vec::new();

        for x in 0..sample_count {
            wav.push(if (x / period) % 2 == 0 {
                tone_volume
            } else {
                -tone_volume
            });
        }
        audio_queue.queue(&wav);
    } else {
        audio_queue.clear();
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    // Setup SDL
    let sdl_context = sdl2::init().unwrap();
    // video_subsystem
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    // audio_subsystem
    let audio_subsystem = sdl_context.audio().unwrap();
    let mut audio_queue = audio_subsystem
        .open_queue::<i16, _>(
            None,
            &AudioSpecDesired {
                freq: Some(48_000),
                channels: Some(2),
                samples: None, // default samples
            },
        )
        .unwrap();
    audio_queue.resume();

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8 = Emu::new();

    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8.load(&buffer);

    let frame = Duration::from_millis((1000 / MCYCLE) as u64);
    'gameloop: loop {
        let now = Instant::now();

        for _ in 0..TICKS_PER_FRAME {
            for evt in event_pump.poll_iter() {
                match evt {
                    Event::Quit { .. } => {
                        break 'gameloop;
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Some(k) = key2btn(key) {
                            chip8.keypress(k, true);
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Some(k) = key2btn(key) {
                            chip8.keypress(k, false);
                        }
                    }
                    _ => (),
                }
            }

            chip8.tick();
            play_audio(&chip8, &mut audio_queue);
        }

        chip8.tick_timers();
        draw_screen(&chip8, &mut canvas);

        if let Some(remaining) = frame.checked_sub(now.elapsed()) {
            sleep(remaining);
        }
    }
}
