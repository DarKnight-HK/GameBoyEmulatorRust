use minifb::{Key, Scale, Window, WindowOptions};
mod bus;
mod cartridge;
mod cpu;
mod dma;
mod interrupts;
mod ppu;
mod timer;

use bus::Bus;
use cartridge::Cartridge;
use cpu::Cpu;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() {
    let mut window = Window::new(
        "Rust GB - Tetris",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.set_target_fps(10);

    // format: 0x00RRGGBB (Red, Green, Blue)
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let bus = Bus::new(Cartridge::new("tetris.gb").unwrap());
    let mut cpu = Cpu::new(bus);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        cpu.step();

        for pixel in buffer.iter_mut() {
            let val = rand::random::<u8>() as u32;
            *pixel = (val << 16) | (val << 8) | val;
        }
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
