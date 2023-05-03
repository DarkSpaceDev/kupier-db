#[macro_use]
extern crate serde_derive;

pub mod db;
pub mod err;
pub mod kupier;
pub mod storage;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
