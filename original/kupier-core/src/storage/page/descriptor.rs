const DESCRIPTOR_PADDING: usize = 9;
const NUM_DESCRIPTOR_SINGLE_BYTES: u32 = 1;
const NUM_DESCRIPTOR_INT_16: u32 = 1;
const NUM_DESCRIPTOR_INT_32: u32 = 1;
const NUM_DESCRIPTOR_INT_64: u32 = 2;

/// The header block of a page
#[derive(Serialize, Deserialize)]
pub struct Descriptor {
    /// Indicates what kind of page this is
    pub page_type: u8,

    /// How big in bytes this page is, header excluded.
    pub page_size: u16,

    /// The file offset to the previous linked page
    pub prev_page_start: u64,

    /// The file offset to the next linked page
    pub next_page_start: u64,

    /// The crc value of this page
    pub crc: u32,

    /// 41 bytes of reserved space - future use only
    reserved: [u8; DESCRIPTOR_PADDING]
}

impl Descriptor {
    pub const HEADER_SIZE: u32 =
        NUM_DESCRIPTOR_SINGLE_BYTES
        + (2 * NUM_DESCRIPTOR_INT_16)
        + (4 * NUM_DESCRIPTOR_INT_32)
        + (8 * NUM_DESCRIPTOR_INT_64)
        + DESCRIPTOR_PADDING as u32;

    pub fn new(page_type: u8,
               page_size: u16,
               prev_page_start: u64,
               next_page_start: u64) -> Descriptor {

        assert_eq!(page_size % 8, 0, "page_size must be divisible by 2");

        Descriptor {
            page_type,
            page_size,
            prev_page_start,
            next_page_start,
            crc: 0,
            reserved: [0xF; DESCRIPTOR_PADDING]
        }
    }
}