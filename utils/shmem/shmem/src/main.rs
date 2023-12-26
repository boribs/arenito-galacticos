// Prueba de sincronización entre procesos.

use shared_memory::*;

// https://doc.rust-lang.org/reference/types/pointer.html
struct AISimMem {
    sync_byte: *mut u8,
    space_mem: *mut u8,
}

impl AISimMem {
    const CAN_WRITE_RUST: u8 = 45;
    const CAN_WRITE_PYTHON: u8 = 75;
    const END: u8 = 100;

    pub fn new(shmem: &Shmem) -> Self {
        unsafe {
            let ptr = shmem.as_ptr();

            Self {
                sync_byte: ptr,
                space_mem: ptr.add(1),
            }
        }
    }

    pub fn can_write(&self) -> bool {
        unsafe {
            *self.sync_byte == AISimMem::CAN_WRITE_RUST
        }
    }

    pub fn done_writing(&mut self) {
        unsafe {
            *self.sync_byte = AISimMem::CAN_WRITE_PYTHON;
        }
    }

    pub fn signal_end(&mut self) {
        unsafe {
            *self.sync_byte = AISimMem::END;
        }
    }

    pub fn write_byte(&mut self, val: u8) {
        unsafe {
            *self.space_mem = val;
        }
    }

    /// Reads byte modified by python process
    pub fn read_byte(&self) -> u8 {
        unsafe {
            *self.space_mem.add(1)
        }
    }
}

fn listen_and_react(aisim: &mut AISimMem, val: &mut u8, iter: &mut u32) {
    if aisim.can_write() {
        aisim.write_byte(*val);

        println!("python left val: {}", aisim.read_byte());

        *val = (*val + 1) % 255;
        *iter += 1;

        // after its done writing all data
        aisim.done_writing();
    }
}

fn main() {
    let flink = "shmem_test";
    let shmem: Shmem = match ShmemConf::new().size(100).flink(flink).create() {
        Ok(m) => {
            println!("created successfully");
            m
        },
        Err(ShmemError::LinkExists) => {
            println!("already exists. connecting.");
            ShmemConf::new().size(100).flink(flink).open().unwrap()
        },
        Err(_) => panic!("you did something very wrong."),
    };

    let mut aisim = AISimMem::new(&shmem);
    let mut val = 0;
    let mut iter: u32 = 0;

    while iter < 100 {
        listen_and_react(&mut aisim, &mut val, &mut iter);
    }

    aisim.signal_end();
    std::thread::sleep(std::time::Duration::from_secs(1));
}
