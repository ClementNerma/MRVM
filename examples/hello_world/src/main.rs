use rand::Rng;
use mrvm::board::{MotherBoard, Bus};
use mrvm_aux::storage::BootROM;
use mrvm_aux::memory::VolatileMem;
use mrvm_aux::display::BufferedDisplay;
use mrvm_tools::lasm::assemble_words;

fn main() {
    println!("> Assembling LASM code...");

    let program = assemble_words(include_str!("display.lasm")).unwrap();

    println!("> Preparing components and motherboard...");

    let mut rng = rand::thread_rng();

    let components: Vec<Box<dyn Bus>> = vec![
        Box::new(BootROM::with_size(program, 0x1000, rng.gen()).unwrap()),
        Box::new(VolatileMem::new(0x1000, rng.gen()).unwrap()),
        Box::new(BufferedDisplay::new(0x100, Box::new(
            |string| println!("[Display] {}", string.unwrap_or("<invalid input received>"))
        ), rng.gen()).unwrap())
    ];

    let mut motherboard = MotherBoard::new(components);

    motherboard.map(|mut mem| {
        mem.map_contiguous(0x00000000, [ 0, 1, 2 ]).mapping.unwrap();
    });
    
    motherboard.reset();

    println!("> Running the program...");

    let cpu = motherboard.cpu();

    while !cpu.halted() {
        let was_at = cpu.regs.pc;
        cpu.next().expect(&format!("Exception occurred at address {:#010X}", was_at));
    }

    println!("> CPU halted.");
}