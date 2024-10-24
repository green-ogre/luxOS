use crate::println;

#[macro_export]
macro_rules! test_case {
    ($name:ident, $blck:block) => {
        paste::paste! {
            #[test_case]
            #[allow(non_upper_case_globals)]
            static  [<__ $name>] : $crate::test::TestFn = $crate::test::TestFn {
                func: &[<__fn_ $name>],
                name: stringify!($name),
                module_path: module_path!(),
            };

            fn [<__fn_ $name>] () -> $crate::test::TestResult {
                #[allow(unused_imports)]
                use $crate::{test_assert, test_assert_eq};
                $blck
                $crate::test::TestResult::Success
            }
        }
    };
}

#[macro_export]
macro_rules! test_assert {
    ($e:expr) => {
        if $e {
        } else {
            return $crate::test::TestResult::Failure(line!() as usize);
        }
    };
}

#[macro_export]
macro_rules! test_assert_eq {
    ($e1:expr, $e2:expr) => {
        $crate::test_assert!($e1 == $e2)
    };
}

pub struct TestFn {
    pub func: &'static dyn Fn() -> TestResult,
    pub name: &'static str,
    pub module_path: &'static str,
}

unsafe impl Sync for TestFn {}

#[derive(PartialEq, Eq)]
pub enum TestResult {
    Success,
    Failure(usize),
}

pub fn test_runner(tests: &[&TestFn]) {
    use crate::exit::{exit_qemu, QemuExitCode};
    use alloc::vec::Vec;

    println!("Running {} tests...\n", tests.len());
    let mut results = Vec::with_capacity(tests.len());
    for test in tests {
        let result = (test.func)();
        let result_msg = match result {
            TestResult::Success => "\x1b[32mOK\x1b[00m",
            TestResult::Failure(_) => "\x1b[31mERR\x1b[00m",
        };
        println!("{}::{} ... [{}]", test.module_path, test.name, result_msg);
        results.push((result, test.name));
    }

    if results.iter().any(|(r, _)| *r != TestResult::Success) {
        let num_pass = results
            .iter()
            .filter(|(r, _)| *r == TestResult::Success)
            .count();
        let num_fail = results
            .iter()
            .filter(|(r, _)| *r != TestResult::Success)
            .count();
        println!(
            "\n\x1b[32m{}\x1b[00m tests \x1b[32mpassed\x1b[00m",
            num_pass
        );
        println!("\x1b[31m{}\x1b[00m tests \x1b[31mfailed\x1b[00m", num_fail);

        for (test, test_name) in results.iter() {
            match test {
                TestResult::Success => {}
                TestResult::Failure(line) => {
                    println!("\t{} failed on line {}", test_name, line,);
                }
            }
        }
    } else {
        println!("\nAll tests \x1b[32mpassed\x1b[00m");
    }

    exit_qemu(QemuExitCode::Success);
}
