use shared_memory::*;

// https://doc.rust-lang.org/reference/types/pointer.html
fn listen_and_react(addr: *mut u8, val: &mut u8, iter: &mut u32) {
    unsafe {
        if *addr == 45 {
            let space_mem = addr.add(1);
            *space_mem = *val;

            println!("python left val: {}", *addr.add(2));
            // std::thread::sleep(std::time::Duration::from_millis(100));

            // after its done writing all data
            *addr = 75;
            *val = (*val + 1) % 255;
            *iter += 1;
        }
    }
}

fn signal_end(addr: *mut u8) {
    unsafe {
        *addr = 100;
    }
}

fn main() {
    let shmem_flink = "shmem_test_2";
    let shmem = ShmemConf::new().size(5000).flink(shmem_flink).create().unwrap();

    println!("10 seconds");
    std::thread::sleep(std::time::Duration::from_secs(10));

    let ptr = shmem.as_ptr();
    let mut val = 0;
    let mut iter: u32 = 0;

    while iter < 100_000_000 {
        listen_and_react(ptr, &mut val, &mut iter);
    }

    signal_end(ptr);
    println!("giving a few seconds for python to react.");
    std::thread::sleep(std::time::Duration::from_secs(1));
}
