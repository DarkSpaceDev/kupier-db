use std::io::{Cursor, Read, Write};
use bson::Document;
use crate::storage::page::Page;

pub struct DataPage {
    page: Page
}

impl DataPage {
    /*
    fn add_document(&mut self, doc: Document) {
        //let doc =  doc! { "test": Bson::String(String::from("test")) };
        let mut byte_array : Vec<u8> = vec![];
        doc.to_writer(&mut byte_array);
        self.page.data.append(&mut byte_array);
    }

    fn read_document_size(&mut self, start: u64) -> i32 {
        const read_size: usize = 4;
        let mut buf= [0; read_size];

        let my_ref = self.page.data.by_ref();
        let mut cursor = Cursor::new(my_ref);
        cursor.set_position(start);
        cursor.read_exact(&mut buf);

        // BSON Format is Little Endian
        // with the exception of timestamp and counter
        // from_be_bytes -> Big Endian
        // from_le_bytes -> Little Endian
        // from_ne_bytes -> Native Endian
        i32::from_le_bytes(buf)
    }

    fn scan_documents2(&mut self) {
        println!("Scan Start ...");

        let data_len = self.page.data.len();

        const read_size: usize = 4;
        let mut sz_buf = [0; read_size];

        // No data to scan ...
        if self.page.data.len() == 0 {
            return;
        }

        let mut num = 0;
        let mut current_pos: u64 = 0;
        let data_ref = self.page.data.by_ref();
        let mut data_cursor = Cursor::new(data_ref);

        println!("Data Size: `{}` bytes", data_len);

        loop {
            if current_pos == data_len as u64 {
                println!("DONE!");
                break;
            }

            if current_pos as u64 > data_len as u64 {
                println!("WTF: Corrupt Page!");
                break;
            }

            data_cursor.set_position(current_pos);
            data_cursor.read_exact(&mut sz_buf);

            let this_len = i32::from_le_bytes(sz_buf);
            current_pos += this_len as u64;
            num += 1;

            println!("[{}] {} bytes", num, this_len);
        }

        println!("`{}` MB scanned", current_pos as f64 / 1024.0 / 1024.0);
    }

    fn scan_documents(&mut self) {
        println!("Scan Start ...");

        // No data to scan ...
        if self.page.data.len() == 0 {
            return;
        }

        let mut num = 0;
        let mut current_pos: u64 = 0;

        println!("Data Size: `{}` bytes", self.page.data.len());

        loop {
            if current_pos == self.page.data.len() as u64 {
                println!("DONE!");
                break;
            }

            if current_pos as u64 > self.page.data.len() as u64 {
                println!("WTF: Corrupt Page!");
                break;
            }

            let mut this_len = self.read_document_size(current_pos);
            current_pos += this_len as u64;
            num += 1;

            println!("[{}] {} bytes", num, this_len);
        }

        println!("`{}` MB scanned", current_pos as f64 / 1024.0 / 1024.0);
    }
    */
}