use crate::info;
use crate::pci::BusDeviceFunction;
use crate::pci::Pci;
use crate::pci::VendorDeviceId;
use crate::result::Result;

pub struct PciXhciDriver {}
impl PciXhciDriver {
    pub fn supports(vp: VendorDeviceId) -> bool {
        const VDI_LIST: [VendorDeviceId; 3] = [
            VendorDeviceId {
                vendor: 0x1b36,
                device: 0x000d,
            },
            VendorDeviceId {
                vendor: 0x8086,
                device: 0x31a8,
            },
            VendorDeviceId {
                vendor: 0x8086,
                device: 0x02ed,
            },
        ];
        VDI_LIST.contains(&vp)
    }
    // xHCIコントローラーの初期化
    // xHCIデバイスの基本的なPCI設定（割り込み無効化、バスマスタリング有効化、レジスタアドレス取得）まで
    pub fn attach(pci: &Pci, bdf: BusDeviceFunction) -> Result<()> {
        info!("Xhci found at: {bdf:?}");
        pci.disable_interrupt(bdf)?;
        pci.enable_bus_master(bdf)?;
        let bar0 = pci.try_bar0_mem64(bdf)?;
        info!("xhci: {bar0:?}");
        Err("wip")  // "Work In Progress"（作業中）
    }
}