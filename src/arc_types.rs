#[derive(Debug)]
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

    pub fn pack(x: i16, y: i16) -> i32 {
        (((x as u32) << 16) | (y as u32 & 0xFFFF)) as i32
    }
}
