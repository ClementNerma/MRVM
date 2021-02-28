use mrvm::board::{Bus, MotherBoard};
use mrvm_aux::display::BufferedDisplay;
use mrvm_aux::storage::BootRom;
use mrvm_aux::volatile_mem::Ram;
use mrvm_tools::lasm::assemble_words;
use rand::Rng;

fn main() {
    println!("> Assembling LASM code...");

    let program = assemble_words(include_str!("display.lasm"))
        .unwrap_or_else(|err| panic!("Failed to assemble demo program: {}", err));

    println!("> Preparing components and motherboard...");

    let mut rng = rand::thread_rng();

    let components: Vec<Box<dyn Bus>> = vec![
        Box::new(BootRom::with_size(program, 0x1000, rng.gen()).unwrap()),
        Box::new(Ram::new(0x1000, rng.gen()).unwrap()),
        Box::new(
            BufferedDisplay::new(
                0x100,
                Box::new(|string| {
                    print!("[Display] {}", string.unwrap_or("<invalid input received>"))
                }),
                rng.gen(),
            )
            .unwrap(),
        ),
    ];

    let mut motherboard = MotherBoard::new(components);

    motherboard.map(|mem| {
        mem.map_contiguous(0x0000_0000, [0, 1, 2]).mapping.unwrap();
    });

    motherboard.reset();

    println!("> Running the program...");

    let cpu = motherboard.cpu();

    while !cpu.halted() {
        cpu.next();
    }

    println!("> CPU halted.");
}
