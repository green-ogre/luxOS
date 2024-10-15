use crate::{memory::AllocHeader, serial_println, ALLOCATOR};
use core::{fmt::Debug, sync::atomic::Ordering};

#[cfg(test)]
pub fn test_runner(tests: &[&dyn Fn()]) {
    use crate::{
        exit::{exit_qemu, QemuExitCode},
        serial_println,
    };

    serial_println!("Running {} tests\n", tests.len());
    for test in tests {
        test();
    }
    serial_println!("\nAll tests passed...");
    exit_qemu(QemuExitCode::Success);
}

#[allow(unused)]
fn assert_all_allocations_vacant() {
    let first_header_ptr = ALLOCATOR.first_header.load(Ordering::Relaxed);
    let mut current_header = unsafe { *(first_header_ptr as *mut AllocHeader) };
    let mut current_address;

    loop {
        assert!(!current_header.is_occupied());

        if !current_header.next_header_is_valid() {
            break;
        } else {
            current_address = current_header.next_header_addr() as *mut u8;
            current_header = unsafe { *(current_address as *mut AllocHeader) };
        }
    }
}

#[allow(unused)]
fn assert_atleast_one_allocation() {
    let first_header_ptr = ALLOCATOR.first_header.load(Ordering::Relaxed);
    let mut current_header = unsafe { *(first_header_ptr as *mut AllocHeader) };
    let mut current_address;
    let mut has_atleast_one_allocation = false;

    loop {
        if current_header.is_occupied() {
            has_atleast_one_allocation = true;
        }

        if !current_header.next_header_is_valid() {
            break;
        } else {
            current_address = current_header.next_header_addr() as *mut u8;
            current_header = unsafe { *(current_address as *mut AllocHeader) };
        }
    }

    if !has_atleast_one_allocation {
        panic!("assert_atleast_one_allocation");
    }
}

#[allow(unused)]
fn assert_num_allocations(num: usize) {
    let first_header_ptr = ALLOCATOR.first_header.load(Ordering::Relaxed);
    let mut current_header = unsafe { *(first_header_ptr as *mut AllocHeader) };
    let mut current_address;
    let mut num_allocations = 0;

    loop {
        if current_header.is_occupied() {
            num_allocations += 1;
        }

        if !current_header.next_header_is_valid() {
            break;
        } else {
            current_address = current_header.next_header_addr() as *mut u8;
            current_header = unsafe { *(current_address as *mut AllocHeader) };
        }
    }

    assert_eq!(num, num_allocations);
}

#[allow(unused)]
fn simple_vec_alloc_dealloc<T: PartialEq + Debug + Clone>(expected_allocations: usize, args: &[T]) {
    {
        let v = args.to_vec();
        serial_println!("{:?}", v);
        for (i, arg) in args.iter().enumerate() {
            assert_eq!(*arg, v[i]);
        }
        assert_atleast_one_allocation();
        assert_num_allocations(expected_allocations);
    }
    assert_all_allocations_vacant();
}

#[test_case]
fn simple_allocation_deallocation() {
    use alloc::{boxed::Box, string::ToString, vec};

    simple_vec_alloc_dealloc(1, &[1, 2, 3]);
    simple_vec_alloc_dealloc(1, &["Hello, ", "World!"]);

    {
        let v = vec!["Hello, ".to_string(), "World!".to_string()];
        serial_println!("{:?}", v);
        assert_eq!("Hello, ", v[0]);
        assert_eq!("World!", v[1]);
        assert_atleast_one_allocation();
        assert_num_allocations(3);
        drop(v);
        assert_all_allocations_vacant();
    }

    {
        let b = Box::new(69);
        serial_println!("{:?}", b);
        assert_eq!(*b, 69);
        assert_num_allocations(1);
        drop(b);
        assert_all_allocations_vacant();
    }

    {
        let mut v = vec!["Hello, ".to_string(), "World!".to_string()];
        serial_println!("{:?}", v);
        assert_eq!("Hello, ", v[0]);
        assert_eq!("World!", v[1]);
        assert_num_allocations(3);

        let b = Box::new(69);
        serial_println!("{:?}", b);
        assert_eq!(*b, 69);
        assert_num_allocations(4);

        serial_println!("{:?}", v);
        assert_eq!("Hello, ", v[0]);
        assert_eq!("World!", v[1]);

        drop(b);
        v.push("My, My".to_string());
        assert_num_allocations(4);
        serial_println!("{:?}", v);
        assert_eq!("Hello, ", v[0]);
        assert_eq!("World!", v[1]);
        assert_eq!("My, My", v[2]);
    }
    assert_all_allocations_vacant();
}
