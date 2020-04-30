use core::alloc::*;
use core::mem;

use core::ptr;

use gba;

pub const EWRAM_BASE: usize = 0x200_0000;
pub const EWRAM_END: usize = 0x203_3FF0;
// Something seems to be in 0x203FFF
// pub const EWRAM_END: usize = 0x203_FFF0;
pub const EWRAM_SIZE: usize = EWRAM_END - EWRAM_BASE;
pub struct MyBigAllocator; // Empty struct; doesn't actually save any data, it's just a handle that can be made immutably `static` to satisfy the global_alloc interface

#[derive(Debug, Clone)]
#[repr(C)]
pub struct BlockAllocate {
    size: usize,  // 4Bytes
    marker: u16,  // 2Bytes
    free: bool,   // 1Byte
    filler: bool, // 1Byte
}

unsafe impl GlobalAlloc for MyBigAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        gba::info!("Alloc Call, find {} Bytes", layout.size());
        // layout.size bytes of data + control bytes + padding for alignment
        let needed_bytes: usize = layout.size()
            + mem::size_of::<BlockAllocate>()
            + if layout.size() % 4 == 0 {
                0
            } else {
                (4 - layout.size() % 4)
            };

        // Current block we're checking for allocation eligibility
        let mut current_block_position = EWRAM_BASE;
        while current_block_position < EWRAM_END {
            gba::info!("Check Block on position 0x{:x}", current_block_position);
            // Obtain a reference to the block we're checking
            let mut current_block: BlockAllocate =
                ptr::read_volatile::<BlockAllocate>(current_block_position as *const BlockAllocate);
            gba::info!("Block Metadata {:?}", current_block);
            assert!(current_block.size != 0);
            // Check whether data + control structure would fit
            if current_block.free && current_block.size >= needed_bytes {
                // big enough free block found
                // lets split it
                let old_size = current_block.size;

                // new size is allocated Bytes + size of Control Block\
                current_block.size = needed_bytes;
                current_block.free = false;
                if (old_size - current_block.size >= 16) {
                    gba::info!(
                        "CALLING CREATE NEW BLOCK MEM 0x{:x} SIZE {}",
                        current_block_position + current_block.size,
                        old_size - current_block.size
                    );
                    create_new_block(
                        current_block_position + current_block.size,
                        old_size - current_block.size,
                    );
                } else {
                    gba::info!("SPACE LEFT IS TO SMALL TO CREATE NEW BLOCK ");
                    current_block.size = old_size;
                }
                gba::info!(
                    "allocated Block at 0x{:x} with size {}",
                    current_block_position,
                    current_block.size
                );
                ptr::write_volatile::<BlockAllocate>(
                    current_block_position as *mut BlockAllocate,
                    current_block.clone(),
                );

                return (current_block_position + mem::size_of::<BlockAllocate>()) as *mut u8;
            }
            // Check next block
            // We have to move layout.size() + mem::size_of::<BlockAllocate>() positions
            current_block_position = current_block_position + current_block.size;
        }
        // If no block could be found, return a null pointer
        return 0 as *mut u8;
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut current_block: BlockAllocate = ptr::read_volatile::<BlockAllocate>(
            ((ptr as usize) - mem::size_of::<BlockAllocate>()) as *const BlockAllocate,
        );

        gba::info!("Dellalloc Block with Meta {:?}", current_block);

        current_block.free = true;
        ptr::write_volatile::<BlockAllocate>(
            ((ptr as usize) - mem::size_of::<BlockAllocate>()) as *mut BlockAllocate,
            current_block.clone(),
        );

        gba::info!("Deallocation {} Bytes at 0x{:p}", layout.size(), ptr);
        merge_free_blocks(((ptr as usize) - mem::size_of::<BlockAllocate>()) as *mut BlockAllocate);
        //merge_free_blocks(EWRAM_BASE as *mut BlockAllocate);
        // TODO: Check whether adjacent blocks are free and perform coalescing

        let mut current_block2: BlockAllocate = ptr::read_volatile::<BlockAllocate>(
            ((ptr as usize) - mem::size_of::<BlockAllocate>()) as *const BlockAllocate,
        );
        gba::info!("Dealloc Block with Meta (AW) {:?}", current_block2);
    }
}

/// Allocate block of `size` at address `base`
pub fn create_new_block(base: usize, size: usize) {
    gba::info!(
        "New free Block with size {} on Mem: 0x{:x}",
        size.clone(),
        base.clone()
    );
    let control = BlockAllocate {
        size: size,
        marker: 0xDEAD,
        free: true,
        filler: false,
    };
    //let control_block: &mut BlockAllocate = mem::transmute::<usize, &mut BlockAllocate>(base);
    gba::info!("Size Test 1 {}", size);
    gba::info!("Size in Control Block: {}", control.size);

    let c2: BlockAllocate;
    let pointer: *mut BlockAllocate = base as *mut BlockAllocate;
    gba::info!("Pointer To Write Base: {:p}", pointer);
    unsafe {
        ptr::write_volatile(pointer, control);

        c2 = ptr::read_volatile::<BlockAllocate>(pointer);
    }
    gba::info!("Pointer To Write Base (AW): {:p}", pointer);

    gba::info!("C2 Perspective");
    //gba::debug!("Position of size var 0x{:p}", &c2.size);
    gba::info!("Size: {}", c2.size.clone());
    //gba::debug!("Position of free var 0x{:p}", &c2.free);
    gba::info!("Free: {}", c2.free.clone());
    //assert_eq!(c2.size, size);
    //assert_eq!(control_block.size, size);
    //assert_eq!(control.size, c2.size.clone())
}

fn merge_free_blocks(ptr: *mut BlockAllocate) {
    let mut c1: BlockAllocate; // vordere
    let mut c2: BlockAllocate; // hintere
    let mut ptr_to_next_block = ptr as usize;
    unsafe {
        c1 = ptr::read_volatile::<BlockAllocate>(ptr);
    }
    ptr_to_next_block += c1.size;
    if !c1.free {
        return;
    }
    loop {
        if (ptr_to_next_block >= EWRAM_END) {
            gba::warn!("Merged with last Block ");
            assert_eq!(ptr_to_next_block, EWRAM_END);

            break;
        }
        unsafe {
            c2 = ptr::read_volatile::<BlockAllocate>(ptr_to_next_block as *mut BlockAllocate);
        }
        if c2.free {
            c1.size += c2.size;
            ptr_to_next_block += c2.size;
            gba::warn!("Merging two Blocks.. new Block {:?}", c1);

            gba::warn!("Next Block should be at 0x{:x}", ptr_to_next_block);
        } else {
            gba::warn!("Stop Merging, Block c2 is not free");
            gba::warn!("BLOCKINFO {:?}", c2);
            assert_eq!(c2.free, false);
            break;
        }
    }
    unsafe {
        ptr::write_volatile(ptr, c1);
    }
}

#[test_case]
fn test_allocator() {
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;
    // Perform some small allocations and ensure that what we expect was allocated
    gba::debug!("Allocating box 1");
    let test_box: Box<u32> = Box::new(3);
    assert_eq!(*test_box, 3);
    gba::debug!("Finished allocating box 1");

    gba::debug!("Allocating box 2");
    let test_box2: Box<u32> = Box::new(5);
    assert_eq!(*test_box2, 5);
    gba::debug!("Finished allocating box 2");

    gba::debug!("Allocating string 1");
    let str = String::from("FOOFOOOFOOOFOFOFOOFOFOF");
    assert_eq!(str.as_str(), "FOOFOOOFOOOFOFOFOOFOFOF");
    gba::debug!("Finished allocating string 1");

    gba::debug!("Alloc tests passed!");
}

#[test_case]
fn allocator_stress_test() {
    use alloc::boxed::Box;
    use alloc::string::String;
    use alloc::vec::Vec;
    // Perform an allocator "stress test" by continuously allocating and dropping large data structures.
    let mut size_bytes: usize = 100;
    let num_objects_per_round = 10;
    for _s in 0..3 {
        gba::info!("[XXXXXXXXXXXXXXXXXXXX] Allocator stress test");
        let mut all_boxes: Vec<Box<[u8]>> = Vec::new();
        for _i in 0..num_objects_per_round {
            // Hack to ensure we don't blow our stack by not first writing to the stack and then copying to the heap
            let test_vec: Box<[u8]> = vec![0xFF; size_bytes].into_boxed_slice();
            all_boxes.push(test_vec);
        }
        gba::info!(
            "[XXXXXXXXXXXXXXXX] Survived allocation of {} byte objects for {} times",
            size_bytes,
            num_objects_per_round
        );
        size_bytes *= 10;
    }
}
