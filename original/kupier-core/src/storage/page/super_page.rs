use std::fs::File;
use std::io::{Seek, Write};
use std::slice::Iter;
use serde_derive;
use crate::error::{Result};
use crate::storage::page::descriptor::Descriptor;
use crate::storage::page::{DynPage, Page, PageType, Stream};

#[derive(Serialize, Deserialize)]
pub struct SuperPage {
    header: Descriptor,

    #[serde(skip_serializing)]
    pub data: Vec<u8>
}

impl DynPage for SuperPage {
    fn get_data_size(&self) -> usize {
        (self.header.page_size as u32 - Descriptor::HEADER_SIZE) as usize
    }

    fn new(page_size: u16,
           prev_page_start: u64,
           next_page_start: u64) -> Box<dyn DynPage> {
        let mut page = SuperPage {
            header: Descriptor::new(PageType::Super,
                                    page_size,
                                    prev_page_start,
                                    next_page_start),
            data: vec![]
        };

        // Fills a vector with empty data ...
        // page.data.resize((page_size as u32 - Descriptor::HEADER_SIZE) as usize, 0);
        // page.data.fill(0);

        return Box::new(page);
    }

    fn get_descriptor(&self) -> &Descriptor {
        &self.header
    }

    fn get_data_iter(&self) -> Iter<'_, u8> {
        self.data.iter()
    }

    fn get_data(&self) -> &[u8] {
        self.data.as_slice()
    }
}