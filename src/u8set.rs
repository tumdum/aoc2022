use std::fmt::{Debug, Error, Formatter};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct U8Set {
    data: [u64; 4],
    pub max: u8,
}

impl Default for U8Set {
    fn default() -> Self {
        Self {
            data: Default::default(),
            max: 255,
        }
    }
}

impl Debug for U8Set {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "U8Set{{")?;
        for i in 0..=self.max {
            if self.contains(i) {
                write!(f, "{i}, ")?;
            }
        }
        write!(f, "}}")
    }
}

impl U8Set {
    pub fn new(max: u8) -> Self {
        Self {
            data: Default::default(),
            max,
        }
    }

    pub fn contains(&self, value: u8) -> bool {
        let idx = (value / 64) as usize;
        let off = 1 << (value % 64);
        let ret = self.data[idx] & off;
        ret != 0
    }

    pub fn insert(&mut self, value: u8) -> bool {
        let idx = (value / 64) as usize;
        let off = 1 << (value % 64);
        let ret = self.data[idx] & off;
        self.data[idx] |= off;
        ret != 0
    }

    pub fn intersection(&self, other: &Self) -> Self {
        Self {
            data: [
                self.data[0] & other.data[0],
                self.data[1] & other.data[1],
                self.data[2] & other.data[2],
                self.data[3] & other.data[3],
            ],
            ..*self
        }
    }

    pub fn difference(&self, other: &Self) -> Self {
        Self {
            data: [
                self.data[0] & !other.data[0],
                self.data[1] & !other.data[1],
                self.data[2] & !other.data[2],
                self.data[3] & !other.data[3],
            ],
            ..*self
        }
    }

    pub fn iter(&'_ self) -> impl Iterator<Item = u8> + '_ {
        (0u8..=self.max).filter(|i| self.contains(*i))
    }

    pub fn is_empty(&self) -> bool {
        self.data == [0, 0, 0, 0]
    }

    pub fn iter_clone(self) -> impl Iterator<Item = u8> + Clone {
        let s = self;
        (0u8..=self.max).filter(move |i| s.contains(*i))
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }
}

impl FromIterator<u8> for U8Set {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u8>,
    {
        let mut ret = Self::default();
        for i in iter {
            ret.insert(i);
        }
        ret
    }
}

impl<'a> FromIterator<&'a u8> for U8Set {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = &'a u8>,
    {
        let mut ret = Self::default();
        for i in iter {
            ret.insert(*i);
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let s = U8Set::default();
        for i in 0u8..=255 {
            assert!(!s.contains(i));
        }
    }

    #[test]
    fn insert() {
        for i in 0u8..=255 {
            let mut s = U8Set::default();
            assert!(!s.contains(i));
            assert!(!s.insert(i));
            assert!(s.contains(i));
            for j in 0u8..=255 {
                if j != i {
                    assert!(!s.contains(j));
                }
            }
        }
    }

    #[test]
    fn from_iter() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.clone().into_iter().collect();
        for i in 0u8..=255 {
            if v.contains(&i) {
                assert!(s.contains(i));
            } else {
                assert!(!s.contains(i));
            }
        }
    }

    #[test]
    fn iter() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.clone().into_iter().collect();
        let v2: Vec<_> = s.iter().collect();
        assert_eq!(v, v2);
    }

    #[test]
    fn intersection_self() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.into_iter().collect();
        let si = s.intersection(&s);
        assert_eq!(si, s);
    }

    #[test]
    fn intersection_sub() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.into_iter().collect();
        let sub = vec![1u8, 68, 99, 157, 200, 255];
        let s_sub: U8Set = sub.into_iter().collect();
        assert_eq!(s_sub, s.intersection(&s_sub));
        assert_eq!(s_sub, s_sub.intersection(&s));
    }

    #[test]
    fn intersection() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let s: U8Set = v.into_iter().collect();
        let w = vec![1u8, 15, 68, 99, 121, 157, 200, 254, 255];
        let s_sub: U8Set = w.into_iter().collect();
        let i = vec![1u8, 68, 99, 157, 200, 255];
        let si: U8Set = i.into_iter().collect();
        assert_eq!(si, s.intersection(&s_sub));
        assert_eq!(si, s_sub.intersection(&s));
    }

    #[test]
    fn diff() {
        let v = vec![1u8, 22, 68, 99, 129, 157, 200, 201, 255];
        let v: U8Set = v.into_iter().collect();
        let w = vec![1u8, 15, 68, 99, 121, 157, 200, 254, 255];
        let w: U8Set = w.into_iter().collect();
        let ret = vec![22, 129, 201];
        let ret: U8Set = ret.into_iter().collect();
        assert_eq!(ret, v.difference(&w));
    }
}
