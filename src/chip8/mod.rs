use self::processor::Processor;

pub mod processor;

pub struct Core {
    pub processor: Processor,
}

impl Core {
    pub fn new(rom_file_path: &str) -> Self {
        let mut processor = Processor::new();
        processor.init();
        processor.load_rom(rom_file_path);

        Self { processor }
    }
}
