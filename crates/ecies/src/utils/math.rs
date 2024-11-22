use std::fmt::Debug;

#[derive(PartialEq, Debug)]
struct U256 {
    a_0: u128,
    a_1: u128,
}

const MAX_U64: u128 = u64::MAX as u128;

// a_0 * 2^128 + a_1
impl U256 {
    pub fn mul(a: &u128, b: &u128) -> Self {
        let (a_0, a_1) = (a >> 64, a & MAX_U64);
        let (b_0, b_1) = (b >> 64, b & MAX_U64);

        let a_0b_1 = a_0 * b_1;
        let a_1b_0 = a_1 * b_0;
        let mut x_0 = a_0 * b_0;
        x_0 += (a_0b_1 >> 64) + (a_1b_0 >> 64);
        if (a_0b_1 & MAX_U64) + (a_1b_0 & MAX_U64) > MAX_U64 {
            x_0 += 1;
        }
        let mut x_1 = (((a_0b_1 & MAX_U64) + (a_1b_0 & MAX_U64)) & MAX_U64) << 64;
        let r = a_1 * b_1;
        if u128::MAX - x_1 < r {
            x_0 += 1;
            x_1 = (x_1 as i128).wrapping_add(r as i128) as u128;
        } else {
            x_1 += r;
        }
        Self { a_0: x_0, a_1: x_1 }
    }
}

impl From<u128> for U256 {
    fn from(value: u128) -> Self {
        Self { a_0: 0, a_1: value }
    }
}

#[derive(Clone)]
pub struct MontgomerySpace {
    modulus: u128,
    inv: u128,
    r2: u128,
    zero: Montgomery,
}

impl MontgomerySpace {
    pub fn new(modulus: u128) -> Self {
        let (inv, r2) = Self::gen_params(modulus);
        MontgomerySpace {
            modulus,
            inv,
            r2,
            zero: MontgomerySpace {
                modulus,
                inv,
                r2,
                zero: Montgomery(0),
            }
            .new_mont(&0),
        }
    }

    pub fn new_mont(&self, val: &u128) -> Montgomery {
        Montgomery::new(self, val)
    }

    pub fn mul(&self, a: &Montgomery, b: &Montgomery) -> Montgomery {
        Montgomery(Montgomery::reduce(self, &U256::mul(&a.0, &b.0)))
    }

    pub fn add(&self, a: &Montgomery, b: &Montgomery) -> Montgomery {
        let mut res = a.0 as i128;
        res = res.wrapping_add(b.0 as i128);
        res = res.wrapping_sub((self.modulus << 1) as i128);
        if res < 0 {
            res += (self.modulus << 1) as i128;
        }
        Montgomery(res as u128)
    }

    pub fn sub(&self, a: &Montgomery, b: &Montgomery) -> Montgomery {
        let mut res = a.0 as i128 - b.0 as i128;
        if res < 0 {
            res += (self.modulus << 1) as i128;
        }
        Montgomery(res as u128)
    }

    pub fn neg(&self, a: &Montgomery) -> Montgomery {
        self.sub(&self.zero, a)
    }

    pub fn inverse(&self, a: &Montgomery) -> Option<Montgomery> {
        a.inverse(self)
    }

    pub fn to_norm(&self, a: &Montgomery) -> u128 {
        a.to_norm(self)
    }

    pub fn zero(&self) -> Montgomery {
        self.zero.clone()
    }

    fn gen_params(modulus: u128) -> (u128, u128) {
        let mut inv = 1i128;
        for _ in 0..7 {
            inv = inv.wrapping_mul(2 - inv.wrapping_mul(modulus as i128));
        }
        let mut r2 = (1 << 127) % modulus;
        for _ in 127..256 {
            r2 <<= 1;
            if r2 >= modulus {
                r2 -= modulus;
            }
        }
        (inv as u128, r2)
    }
}
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Montgomery(u128);

impl Montgomery {
    fn new(space: &MontgomerySpace, value: &u128) -> Self {
        Montgomery(Self::transform(space, value))
    }

    fn to_norm(&self, space: &MontgomerySpace) -> u128 {
        Montgomery::reduce(space, &U256::from(self.0))
    }

    fn inverse(&self, space: &MontgomerySpace) -> Option<Montgomery> {
        let mut a1 = self.to_norm(space) as i128;
        let mut b1 = space.modulus as i128;
        let mut x = 1;
        let mut y = 0;
        let mut x1 = 0;
        let mut y1 = 1;
        while b1 > 0 {
            let q = a1 / b1;
            (x, x1) = (x1, x - q * x1);
            (y, y1) = (y1, y - q * y1);
            (a1, b1) = (b1, a1 - q * b1);
        }
        if x < 0 {
            x += space.modulus as i128;
        };
        if a1 == 1 {
            Some(Montgomery::new(space, &(x as u128)))
        } else {
            None
        }
    }

    fn reduce(space: &MontgomerySpace, x: &U256) -> u128 {
        let q = U256::mul(&x.a_1, &space.inv).a_1;
        let mut a: i128 = (x.a_0 as i128) - (U256::mul(&q, &space.modulus).a_0 as i128);
        if a < 0 {
            a += space.modulus as i128;
        } else {
            a %= space.modulus as i128;
        }
        a as u128
    }

    fn transform(space: &MontgomerySpace, x: &u128) -> u128 {
        Self::reduce(space, &U256::mul(x, &space.r2))
    }
}

// impl From<&Montgomery> for u128 {
//     fn from(value: &Montgomery) -> Self {
//         Montgomery::reduce(&U256::from(value.0))
//     }
// }
//
// impl Mul<&Montgomery> for &Montgomery {
//     type Output = Montgomery;
//
//     fn mul(self, rhs: &Montgomery) -> Self::Output {
//         Montgomery(Montgomery::reduce(&U256::mul(&self.0, &rhs.0)))
//     }
// }
//
// impl Mul<&Montgomery> for Montgomery {
//     type Output = Montgomery;
//
//     fn mul(self, rhs: &Montgomery) -> Self::Output {
//         &self * rhs
//     }
// }
//
// impl Mul<Montgomery> for Montgomery {
//     type Output = Montgomery;
//
//     fn mul(self, rhs: Montgomery) -> Self::Output {
//         &self * &rhs
//     }
// }
//
// impl Mul<&u128> for &Montgomery {
//     type Output = Montgomery;
//
//     fn mul(self, rhs: &u128) -> Self::Output {
//         self * &Montgomery::new(rhs)
//     }
// }
//
// impl Mul<&u128> for Montgomery {
//     type Output = Montgomery;
//
//     fn mul(self, rhs: &u128) -> Self::Output {
//         &self * &Montgomery::new(rhs)
//     }
// }
//
// impl Add<&Montgomery> for &Montgomery {
//     type Output = Montgomery;
//
//     fn add(self, rhs: &Montgomery) -> Self::Output {
//         let mut res = self.0 as i128;
//         println!("{}", self.0);
//         res = res.wrapping_add(rhs.0 as i128);
//         println!("{}", rhs.0);
//         res = res.wrapping_sub((MODULUS << 1) as i128);
//         if res < 0 {
//             res += (MODULUS << 1) as i128;
//         }
//         Montgomery(res as u128)
//     }
// }
//
// impl Add<&Montgomery> for Montgomery {
//     type Output = Montgomery;
//
//     fn add(self, rhs: &Montgomery) -> Self::Output {
//         &self + rhs
//     }
// }
//
// impl Add<&u128> for &Montgomery {
//     type Output = Montgomery;
//
//     fn add(self, rhs: &u128) -> Self::Output {
//         self + &Montgomery::new(rhs)
//     }
// }
//
// impl Add<&u128> for Montgomery {
//     type Output = Montgomery;
//
//     fn add(self, rhs: &u128) -> Self::Output {
//         &self + &Montgomery::new(rhs)
//     }
// }
//
// impl Sub<&Montgomery> for &Montgomery {
//     type Output = Montgomery;
//
//     fn sub(self, rhs: &Montgomery) -> Self::Output {
//         let mut res = self.0 as i128 - rhs.0 as i128;
//         if res < 0 {
//             res += (MODULUS << 1) as i128;
//         }
//         Montgomery(res as u128)
//     }
// }
//
// impl Sub<Montgomery> for &Montgomery {
//     type Output = Montgomery;
//
//     fn sub(self, rhs: Montgomery) -> Self::Output {
//         self - &rhs
//     }
// }
//
// impl Sub<Montgomery> for Montgomery {
//     type Output = Montgomery;
//
//     fn sub(self, rhs: Montgomery) -> Self::Output {
//         &self - &rhs
//     }
// }
//
// impl Sub<&u128> for &Montgomery {
//     type Output = Montgomery;
//
//     fn sub(self, rhs: &u128) -> Self::Output {
//         self - &Montgomery::new(rhs)
//     }
// }
//
// impl Sub<&u128> for Montgomery {
//     type Output = Montgomery;
//
//     fn sub(self, rhs: &u128) -> Self::Output {
//         &self - &Montgomery::new(rhs)
//     }
// }
//
// impl Neg for &Montgomery {
//     type Output = Montgomery;
//
//     fn neg(self) -> Self::Output {
//         &Montgomery::new(&0) - self
//     }
// }

#[cfg(test)]
mod tests {
    use crate::utils::math::{MontgomerySpace, U256};

    #[test]
    fn tt() {
        let s = MontgomerySpace::new(0xFFFF_FFFF_0000_0001);
        dbg!(s.to_norm(&s.inverse(&s.new_mont(&343)).unwrap()));
        // println!("{:?}", U256::mul(&312736963107901427148245483844828457770, &1000000007));
        // println!("{:?}", (&Montgomery::new(&123456789) + &123456789123456789).to_norm());
        // println!("{}", Montgomery::new(&12345).inverse().unwrap().to_norm())
        // println!("{}", g.reduce(&U256::from(936536506)));
        // println!("{:?}", U256::mul(&((1 << 100) + 236842), &((1 << 100) + 14514)));
    }

    #[test]
    fn test_u256() {
        assert_eq!(
            U256::mul(
                &338499862550071182731788823806128222936,
                &226854911280625642308916404954512115491
            ),
            U256 {
                a_0: 225666575033380788487859215870752123277,
                a_1: 273489856730187794420873600817111440264,
            }
        );
    }

    #[test]
    fn test_gen_params() {
        assert_eq!(
            MontgomerySpace::gen_params(1000000007),
            (97035725200851971538472047020383699895, 792845266)
        );
    }
}
