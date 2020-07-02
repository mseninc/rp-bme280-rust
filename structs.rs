pub struct CalibParams {
    pub temperature: [i32; 3],
    pub pressure: [i32; 9],
    pub humidity: [i32; 6],
}

pub struct EnvData {
    pub temperature: u32,
    pub pressure: u32,
    pub humidity: u32,
}
