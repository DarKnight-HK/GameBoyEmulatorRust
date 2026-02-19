use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
mod bus;
mod cartridge;
mod cpu;
mod dma;
mod interrupts;
mod joypad;
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

    window.set_target_fps(60);

    let bus = Bus::new(Cartridge::new("aip.gb").unwrap());
    let mut cpu = Cpu::new(bus);

    const CYCLES_PER_FRAME: u32 = 70224;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let inputs = [
            (7, window.is_key_pressed(Key::Enter, KeyRepeat::No)),
            (6, window.is_key_pressed(Key::Space, KeyRepeat::No)),
            (5, window.is_key_down(Key::Z)),
            (4, window.is_key_down(Key::X)),
            (3, window.is_key_down(Key::Down)),
            (2, window.is_key_down(Key::Up)),
            (1, window.is_key_down(Key::Left)),
            (0, window.is_key_down(Key::Right)),
        ];

        for (bit, pressed) in inputs {
            let request_int = cpu.bus.joypad.set_button(bit, pressed);
            if request_int {
                cpu.bus.request_interrupt(interrupts::Interrupt::Joypad);
            }
        }
        let mut cycles_this_frame = 0;

        while cycles_this_frame < CYCLES_PER_FRAME {
            let cycles = cpu.step() as u32;
            cycles_this_frame += cycles;

            cpu.bus.tick(cycles as u8);

            cpu.check_interrupts();
        }

        window
            .update_with_buffer(&cpu.bus.ppu.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
