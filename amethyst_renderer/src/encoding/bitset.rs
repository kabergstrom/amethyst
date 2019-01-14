use hibitset::{BitSet, BitSetAnd, BitSetLike};

/// A bitset that ANDs a dynamic list of bitsets
pub struct VecBitSet<'a>(pub Vec<&'a BitSet>);

impl<'a> BitSetLike for VecBitSet<'a> {
    #[inline]
    fn layer3(&self) -> usize {
        self.0.iter().fold(0, |x, s| x & s.layer3())
    }
    #[inline]
    fn layer2(&self, id: usize) -> usize {
        self.0.iter().fold(0, |x, s| x & s.layer2(id))
    }
    #[inline]
    fn layer1(&self, id: usize) -> usize {
        self.0.iter().fold(0, |x, s| x & s.layer1(id))
    }
    #[inline]
    fn layer0(&self, id: usize) -> usize {
        self.0.iter().fold(0, |x, s| x & s.layer0(id))
    }
    #[inline]
    fn contains(&self, i: u32) -> bool {
        self.0.iter().fold(true, |x, s| x && s.contains(i))
    }
}

// idea: optimize vec bitsets up to certain length for efficient iteration

// pub enum OptimizedBitSet<'a> {
//     One(&'a BitSet),
//     Two(BitSetAnd<&'a BitSet, &'a BitSet>),
//     Three(BitSetAnd<&'a BitSet, BitSetAnd<&'a BitSet, &'a BitSet>>),
//     Four(BitSetAnd<BitSetAnd<&'a BitSet, &'a BitSet>, BitSetAnd<&'a BitSet, &'a BitSet>>),
// }

// maybe `dyn BitSetLike` would be better?

// impl<'a> VecBitSet<'a> {
//     fn optimize() {}
// }
