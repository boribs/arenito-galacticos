use memmap;
use std::fs::OpenOptions;

fn main() {
    let src = "Hello!";

    let mut f = OpenOptions::new()
        .read(true)
        .write(true)
        //.create(true)
        .open("..//file")
        .expect("Unable to open file");

    let mut data = unsafe {
        memmap::MmapOptions::new()
            .map_mut(&f)
            .expect("Could not access data from memory mapped file")
    };

        for i in 0..10 {
            data[i] = b'k';
        }


    println!("{}", data[0] as char);
}
