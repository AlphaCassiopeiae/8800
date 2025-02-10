
// s100 device(main construct?)
pub struct Device {
    start_addr: u16,
    end_addr: u16,
}

impl Device {
    pub fn new(start_addr: u16, end_addr: u16) -> Self {
        Device { start_addr, end_addr }
    }

    pub fn is_addr_in_range(&self, addr: u16) -> bool {
        addr >= self.start_addr && addr <= self.end_addr
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_addr_range() {
        let device = Device::new(100, 200);
        assert!(device.is_addr_in_range(150));
        assert!(!device.is_addr_in_range(50));
        assert!(!device.is_addr_in_range(250));
    }
}


