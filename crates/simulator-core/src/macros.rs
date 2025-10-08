#[macro_export]
macro_rules! impl_set_range {
    ( $( $flags:ty ),+ ) => {
        $(
            impl $flags {
                #[inline]
                pub fn set_range(&mut self, pos: u8, range: u8) {
                    let mask = (((1 << range) - 1) << pos) as <$flags as bitflags::Flags>::Bits;
                    let new_flags = <$flags>::from_bits_truncate(self.bits() | mask);
                    *self = new_flags;
                }
            }
        )+
    };
}
