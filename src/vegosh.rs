
const TABLE_SIZE: usize = 1 << 21;
const ALIGNMENT: usize = 64;

const EMPTY: u8 = 0x00;
const OCCUPIED: u8 = 0x01;
const TOMBSTONE: u8 = 0x02;

#[repr(C, align(64))]
pub struct Slot {
    key: [u8; 16],
    value: [u8; 32],
    status: u8,
    probe_distance: u8,
    hash: u64,
    padding: [u8; 6],
}

#[repr(C, align(64))]
struct Vegosh([Slot; TABLE_SIZE]);

static mut VEGOSH: Vegosh = Vegosh([Slot {
    key: [0u8; 16],
    value: [0u8; 32],
    status: EMPTY,
    probe_distance: 0,
    hash: 0,
    padding: [0u8; 6],
}; TABLE_SIZE]);


fn initialize_vegosh() {
    unsafe {
        assert!(
            std::mem::size_of::<Slot>() == 64,
            "Slot must be exactly 64 bytes (one cache line)"
        );
        assert!(
            std::mem::size_of::<Slot>() % 64 == 0,
            "Slot size must be a multiple of 64 bytes"
        );

        let ptr = VEGOSH.0.as_mut_ptr() as *mut u8;
        let total_size = std::mem::size_of::<HashTable>();

        std::ptr::write_bytes(ptr, 0, total_size);

        println!(
            "Allocated {} bytes at {:p}",
            total_size,
            ptr
        );
    }
}
