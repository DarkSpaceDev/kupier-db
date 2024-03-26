//--------------------------------------------------------------------------
// (C) Copyright Travis Sharp <travis@darkspace.dev>.  All rights reserved.
//--------------------------------------------------------------------------

#[allow(unused_imports)]
#[macro_use]
extern crate serde_derive;

pub mod error;
pub mod kuiper;
pub mod schema;
pub mod storage;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
