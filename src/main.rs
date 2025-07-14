#![no_std] // stdクレートは使わないという強い意志。
#![no_main]
// no_stdだとmain()関数がstart(どの関数をはじめに実行するかを指定)の役割を果たしてる。
#![feature(offset_of)]

use core::arch::asm; // HLT命令を呼び出す関数をインラインアセンブリで記述したい
use core::cmp::min;
use core::mem::offset_of;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use core::slice;

type EfiVoid = u8;
type EfiHandle = u64;
type Result<T> = core::result::Result<T, &'static str>;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct EfiGuid {
    pub data0: u32,
    pub data1: u16,
    pub data2: u16,
    pub data3: [u8; 8],
}
// UEFI仕様書に書いてある「EFI Graphics Output Protocol」のGUIDの値
const EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID: EfiGuid = EfiGuid {
    data0: 0x9042a9de,
    data1: 0x23dc,
    data2: 0x4a38,
    data3: [0x96, 0xfb, 0x7a, 0xde, 0xd0, 0x80, 0x51, 0x6a],
};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[must_use]
#[repr(u64)]
enum EfiStatus {
    Success = 0,
}

#[repr(C)]
struct EfiBootServicesTable {
    _reserved0: [u64; 40],
    locate_protocol: extern "win64" fn(
        protocol: *const EfiGuid,
        registration: *const EfiVoid,
        interface: *mut *mut EfiVoid,
    ) -> EfiStatus,
}
const _: () = assert!(offset_of!(EfiBootServicesTable, locate_protocol) == 320);
// efi_main()の第二引数に渡されるEfi System Tableからlocate_protocol()のアドレスを得る
// EFI System Tableの中のEFI Boot Services Tableの中に書かれている
#[repr(C)]
struct EfiSystemTable {
    _reserved0: [u64; 12],
    pub boot_services: &'static EfiBootServicesTable,
}
const _: () = assert!(offset_of!(EfiSystemTable, boot_services) == 96);

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolMode<'a> {
    pub max_mode: u32,
    pub mode: u32,
    pub info: &'a EfiGraphicsOutputProtocolPixelInfo,
    pub size_of_info: u64,
    pub frame_buffer_base: usize, // 画面に表示されるピクセルの情報が並んだフレームバッファの開始アドレス
    pub frame_buffer_size: usize, // フレームバッファのバイト単位での大きさ
}

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutPutProtocol<'a> {
    reserved: [u64; 3],
    pub mode: &'a EfiGraphicsOutputProtocolMode<'a>,
}
fn locate_graphic_protocol<'a>(
    efi_system_table: &EfiSystemTable,
) -> Result<&'a EfiGraphicsOutPutProtocol<'a>> {
    let mut graphic_output_protocol = null_mut::<EfiGraphicsOutPutProtocol>();
    let status = (efi_system_table.boot_services.locate_protocol)(
        &EFI_GRAPHICS_OUTPUT_PROTOCOL_GUID,
        null_mut::<EfiVoid>(),
        &mut graphic_output_protocol as *mut *mut EfiGraphicsOutPutProtocol as *mut *mut EfiVoid,
    );
    if status != EfiStatus::Success {
        return Err("Failed to locate graphics output protocol");
    }
    Ok(unsafe { &*graphic_output_protocol })
}

#[repr(C)]
#[derive(Debug)]
struct EfiGraphicsOutputProtocolPixelInfo {
    version: u32,
    pub horizontal_resolution: u32, // 水平方向の画素数
    pub vertical_resolution: u32,   // 垂直方向の画素数
    _padding0: [u32; 5],
    pub pixels_per_scan_line: u32,
}
const _: () = assert!(size_of::<EfiGraphicsOutputProtocolPixelInfo>() == 36);

pub fn hlt() {
    unsafe { asm!("hlt") }
}

#[no_mangle]
fn efi_main(_image_handle: EfiHandle, efi_system_table: &EfiSystemTable) {
    let mut vram = init_vram(efi_system_table).expect("init_vram failed");
    for y in 0..vram.height {
        for x in 0..vram.width {
            if let Some(pixel) = vram.pixel_at_mut(x, y) {
                *pixel = 0x000ff00;
            }
        }
    }
    for y in 0..vram.height / 2 {
        for x in 0..vram.width / 2 {
            if let Some(pixel) = vram.pixel_at_mut(x, y) {
                *pixel = 0xff0000;
            }
        }
    }

    // println!("Hello, world!");
    loop {
        hlt() // 空のloopだとCPUサイクルを消費してしまうので、HLT命令で割り込みが来るまで休ませる
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hlt() // 空のloopだとCPUサイクルを消費してしまうので、HLT命令で割り込みが来るまで休ませる
    }
}

trait Bitmap {
    fn bytes_per_pixel(&self) -> i64;
    fn pixels_per_line(&self) -> i64;
    fn width(&self) -> i64;
    fn height(&self) -> i64;
    fn buf_mut(&mut self) -> *mut u8;
    /// # Safety
    /// 
    /// Returned pointer is valid as long as the given coordinates are valid which means that passing is_in_*_range tests. 
    /// 返されるポインタは、与えられた座標が有効である限り有効であり、is_in_*_rangeテストをパスすることを意味する。
    unsafe fn unchecked_pixel_at_mut(&mut self, x: i64, y: i64) -> *mut u32 {
        self.buf_mut().add(
            ((y * self.pixels_per_line() + x) * self.bytes_per_pixel()) as usize,
        ) as *mut u32
    }
    fn pixel_at_mut(&mut self, x:i64, y:i64) -> Option<&mut u32> {
        if self.is_in_x_range(x) && self.is_in_y_range(y) {
            // SAFETY: (x,y) is always validated by the checks above. 上記によりx,yは常に安全
            unsafe {Some(&mut *(self.unchecked_pixel_at_mut(x, y)))}
        } else {
            None
        }
    }
    fn is_in_x_range(&self, px:i64) -> bool {
        0 <= px && px < min(self.width(), self.pixels_per_line())
    }
    fn is_in_y_range(&self, py:i64) -> bool {
        0 <= py && py < self.height()
    }
}

#[derive(Clone, Copy)]
struct VramBufferInfo {
    buf: *mut u8,
    width: i64,
    height: i64,
    pixels_per_line: i64,
}

impl Bitmap for VramBufferInfo {
    fn bytes_per_pixel(&self) -> i64 {
        4
    }
    fn pixels_per_line(&self) -> i64 {
        self.pixels_per_line
    }
    fn width(&self) -> i64 {
        self.width
    }
    fn height(&self) -> i64 {
        self.height
    }
    fn buf_mut(&mut self) -> *mut u8 {
        self.buf
    }
}

fn init_vram(efi_system_table: &EfiSystemTable) -> Result<VramBufferInfo> {
    let gp = locate_graphic_protocol(efi_system_table)?;
    Ok(VramBufferInfo { 
        buf: gp.mode.frame_buffer_base as *mut u8, 
        width: gp.mode.info.horizontal_resolution as i64, 
        height: gp.mode.info.vertical_resolution as i64, 
        pixels_per_line: gp.mode.info.pixels_per_scan_line as i64,
    })
}
