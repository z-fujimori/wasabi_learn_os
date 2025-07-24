use crate::x86::hlt;
use crate::x86::write_io_port_u8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x1,
    Failed = 0x2,
}
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    write_io_port_u8(0xf4, exit_code as u8);
    // HLT命令でCPUを休ませる
    loop {
        hlt();
    } // 無限ループで終了
}
