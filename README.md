![Space Invaders](https://i.imgur.com/GytYxLQ.png)
# chip8-emulator
This is a simple CHIP-8 emulator written in Rust.
This project isn't quite finished yet so expect bugs.
You can find various CHIP-8 ROMS on github and elsewhere to use for this emulator.

## Prerequisites
- Rust (1.69.0)
- This emulator also depends on SDL2 for rendering, you can install it here https://www.libsdl.org/.

## Building
1. Clone the repo ```git clone https://github.com/d4n13ls1104/chip8-emulator.git```
2. ```cd``` into the repo folder and run ```cargo build --release```

## Usage
Make sure your ```SDL2.dll``` is in the same directory as the program.
You can now run ROMS by simply opening the files with the program or passing the file path of the ROM as an argument to the program.
