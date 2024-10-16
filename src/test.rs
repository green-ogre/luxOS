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
