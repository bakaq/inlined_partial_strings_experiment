#![allow(dead_code, unused_variables, unused_braces)]

use modular_bitfield::prelude::*;

#[repr(u64)]
#[bitfield]
#[derive(Copy, Clone, Debug)]
pub struct HeapCellValue {
    val: B56,
    f: bool,
    m: bool,
    tag: B6,
}

pub type Heap = Vec<HeapCellValue>;

// TODO: I think that this could be done entirely in safe Rust
pub fn push_raw_str(heap: &mut Heap, string: &str) {
    let num_cells = string.len() / 8 + 1;

    heap.reserve(num_cells);
    let buffer = heap.spare_capacity_mut();

    // Sets the last cell as 0
    buffer[num_cells-1].write(0.into());

    // SAFETY: We have reserved all this memory above
    unsafe { 
        // Copies all the the bytes of the string
        std::ptr::copy(string.as_ptr(), buffer.as_mut_ptr() as *mut u8, string.len());
        heap.set_len(heap.len() + num_cells);
    }

}

/// Reads a inlined string from `heap` at `index`.
///
/// # Safety
///
/// The index must be a valid inlined string.
pub unsafe fn read_raw_str(heap: &Heap, index: usize) -> &str {
    let ref_to_start = heap.as_ptr().offset(index as isize);
    let pointer = ref_to_start as *const i8;
    std::ffi::CStr::from_ptr(pointer).to_str().unwrap()
}

// This is like an alternate Debug implementation for Vec<HeapCellValue>
fn inspect_heap(heap: &Heap) -> String {
    let mut heap_view: String = "[".into();
    heap_view += &heap.iter()
        .map(|x| {
            let number: u64 = (*x).into();
            format!("{:#018X}", number.swap_bytes())
        }).collect::<Vec<_>>()
        .join(", ");
    heap_view += "]";

    format!("{heap_view:?}")
}


#[test]
fn test_push_raw_str() {
    let mut heap: Heap = Vec::new();

    push_raw_str(&mut heap, "");
    assert_eq!(heap.len(), 1);

    push_raw_str(&mut heap, "1234");
    assert_eq!(heap.len(), 2);

    // This should take 2 cells
    push_raw_str(&mut heap, "12345678");
    assert_eq!(heap.len(), 4);

    // This should also take 2 cells
    push_raw_str(&mut heap, "123456789");
    assert_eq!(heap.len(), 6);
}

#[test]
fn test_read_str_raw() {
    let mut heap: Heap = Vec::new();

    push_raw_str(&mut heap, "");
    assert_eq!(heap.len(), 1);
    assert_eq!(unsafe{read_raw_str(&heap, 0)}, "");

    push_raw_str(&mut heap, "1234");
    assert_eq!(heap.len(), 2);
    assert_eq!(unsafe{read_raw_str(&heap, 1)}, "1234");

    // This should take 2 cells
    push_raw_str(&mut heap, "12345678");
    assert_eq!(heap.len(), 4);
    assert_eq!(unsafe{read_raw_str(&heap, 2)}, "12345678");

    // This should also take 2 cells
    push_raw_str(&mut heap, "123456789");
    assert_eq!(heap.len(), 6);
    assert_eq!(unsafe{read_raw_str(&heap, 4)}, "123456789");
}
