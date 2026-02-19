# GameBoyEmulator (Rust)

A hobby Game Boy (DMG) emulator written in Rust.  
It currently focuses on core CPU + memory + rendering behavior for real ROMs, with a desktop window powered by `minifb`.

## Showcase

https://youtu.be/JaFb3U7mIQE

## What this emulator can do right now

- Load and run `.gb` ROM files (DMG/Game Boy classic format).
- Execute a large portion of the LR35902 CPU instruction set (including CB-prefixed ops).
- Emulate core memory regions and bus routing:
  - ROM, VRAM, WRAM, HRAM
  - OAM / DMA transfer
  - I/O registers for timer, joypad, interrupts, and PPU
- Render 160x144 frames at approximately 60 FPS in a desktop window.
- Draw background + sprites with DMG-style 4-shade palette mapping.
- Handle interrupts (VBlank, LCD STAT, Timer, Joypad).
- Emulate timer registers (`DIV`, `TIMA`, `TMA`, `TAC`) with overflow interrupt behavior.
- Support cartridge types:
  - ROM-only
  - MBC1 (ROM banking + RAM banking control)

## Controls

- `Enter` = Start
- `Space` = Select
- `X` = A
- `Z` = B
- Arrow keys = D-pad
- `Esc` = Exit emulator

## What is lacking right now

This emulator is still in progress and is not cycle-perfect. Some games may boot but behave incorrectly.

- **No audio/APU emulation yet**
  - Sound channels are not implemented.
- **No boot ROM emulation**
  - CPU starts from post-boot register defaults.
- **Mapper support is incomplete**
  - MBC2/MBC3 are recognized in header parsing but not emulated.
  - No RTC support (required by many MBC3 games).
- **Timing and hardware-accuracy gaps**
  - Not all edge cases and hardware quirks are implemented.
  - Some titles may show visual glitches or unstable gameplay.
- **PPU fidelity is still basic**
  - Pixel pipeline behavior and fine timing are simplified.
  - Window/layer edge cases are not fully verified.
- **No save persistence yet**
  - External RAM is currently in-memory only (no `.sav` read/write).
- **ROM selection is hardcoded in the app**
  - The ROM path is currently selected inside `src/main.rs`.

## Build & run

Requirements:

- Rust toolchain (stable)

Commands:

```bash
cargo build --release
cargo run --release
```

## Project structure

- `src/cpu.rs` - CPU core and instruction execution
- `src/bus.rs` - memory map and component interconnect
- `src/cartridge.rs` - ROM header parsing and cartridge/MBC behavior
- `src/ppu.rs` - video rendering and LCD registers
- `src/timer.rs` - timer/divider emulation
- `src/joypad.rs` - joypad register and key state handling
- `src/interrupts.rs` - interrupt vectors and masks
- `src/main.rs` - window loop, input polling, frame stepping

## Suggested next milestones

1. Add CLI ROM loading (`cargo run -- path/to/game.gb`) instead of hardcoded path.
2. Implement save RAM persistence (`.sav` files).
3. Add MBC3 (+ RTC) and MBC2 support.
4. Improve PPU timing/pixel pipeline correctness.
5. Add APU/sound emulation.
6. Add test ROM automation (e.g., Blargg/Mooneye subsets) for regressions.
