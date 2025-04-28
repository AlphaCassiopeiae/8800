pub struct RAM<C1: core::ops::AsyncFnMut(u32) -> u8, C2: core::ops::AsyncFnMut(u32, u8)> {
    read_fn: C1,
    write_fn: C2,
}

impl<C1: core::ops::AsyncFnMut(u32) -> u8, C2: core::ops::AsyncFnMut(u32, u8)> RAM<C1, C2> {
    /// Configure PIO for SMEMR# control (pin 34)

    pub async fn new(read_fn: C1, write_fn: C2) -> Self {
        Self { read_fn, write_fn }
    }

    // for simulating instructions being present in memory, do nothing for now
    pub async fn init(&mut self) {
        self.write(0x0, 0x3E).await;
        self.write(0x1, 0x05).await;
        self.write(0x2, 0xC6).await;
        self.write(0x3, 0x07).await;
        self.write(0x4, 0x76).await;
    }

    pub async fn read(&mut self, addr: usize) -> u8 {
        self.read_fn.async_call_mut((addr as u32,)).await
    }

    pub async fn write(&mut self, address: usize, data: u8) {
        self.write_fn.async_call_mut((address as u32, data)).await;
    }

    // pub async fn disable(&mut self,) {
    //     self.write_fn.async_call_mut((0, 0, false)).await;
    // }
}
