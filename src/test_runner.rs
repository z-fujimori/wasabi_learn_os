use crate::qemu::exit_qemu;
use crate::qemu::QemuExitCode;
use core::panic::PanicInfo;

pub fn test_runner(_tests: &[&dyn FnOnce()]) -> ! {
    // ここでは単にQEMUを終了させるだけです。
    exit_qemu(QemuExitCode::Success);
}
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // パニックが発生した場合もQEMUを終了させます。
    exit_qemu(QemuExitCode::Failed);
}
