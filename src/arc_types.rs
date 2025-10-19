#[derive(Debug, Clone)]
pub struct Point2 {
    pub x: i16,
    pub y: i16,
}
impl Point2 {
    pub fn unpack(pos: u32) -> Self {
        let x = ((pos as u32) >> 16) as i16;
        let y = (pos as u32 & 0xFFFF) as i16;
        Point2 { x, y }
    }

    pub fn pack(&self) -> i32 {
        (((self.x as u32) << 16) | (self.y as u32 & 0xFFFF)) as i32
    }
}
