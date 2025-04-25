#[macro_export]
macro_rules! fields_inner {
    (bool $type:ty [$start_bit:expr] [$($bits:tt)*] $read_fn:ident $set_fn:ident $($name:ident)*) => {

        pub fn $read_fn(&self) -> bool {
            self.0 << ($start_bit) >> (Self::NUM_BITS - 1) != 0
        }

        pub fn $set_fn(&mut self, value: bool) {
            let shift = Self::NUM_BITS - 1 - ($start_bit);
            self.0 &= !(1 << shift);
            self.0 |= (value as $type) << shift;
        }

        fields!($type [$start_bit + 1] [$($bits)*] $($name)*);
    };
    (u8 $type:ty [$start_bit:expr] [$size:literal $($bits:tt)*] $read_fn:ident $set_fn:ident $($name:ident)*) => {

        pub fn $read_fn(&self) -> u8 {
            (self.0 << ($start_bit) >> (Self::NUM_BITS - $size)) as u8
        }

        pub fn $set_fn(&mut self, value: u8) {
            let shift = Self::NUM_BITS - ($start_bit) - $size;
            self.0 &= !(!(!0 << ($size)) << shift);
            self.0 |= (value as $type) << shift;
        }

        fields!($type [$start_bit + $size] [$($bits)*] $($name)*);
    }
}

macro_rules! fields {
    ($type:ty [$start_bit:expr] []) => {};
    ($type:ty [$start_bit:expr] [1 $($bits:tt)*] $($name:ident)*) => {
        fields_inner!(bool $type [$start_bit] [$($bits)*] $($name)*);
    };
    ($type:ty [$start_bit:expr] [$($bits:tt)*] $($name:ident)*) => {
        fields_inner!(u8 $type [$start_bit] [$($bits)*] $($name)*);
    };
}

macro_rules! bitfield {
    ($name:ident($type:ty); $($bits:tt)|+ $($read_fn:ident $set_fn:ident)+) => {
        
        #[derive(Debug)]
        pub struct $name($type);

        impl $name {
            const NUM_BITS: usize = std::mem::size_of::<$type>() * 8;
            pub fn default() -> Self {
                Self(0)
            }
            fields!($type [0] [$($bits)+] $($read_fn $set_fn)+);
        }
    }
}