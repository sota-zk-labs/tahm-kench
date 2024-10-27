use crate::ecc::point::Point;
use crate::public_key::PublicKey;
use crate::utils::math::{Montgomery, MontgomerySpace};
// curve: y^2 = x^3 + ax + b
pub struct Ecc {
    mont_space: MontgomerySpace,
    // modulus: u128,
    a: Montgomery,
    // b: Montgomery,
    g: Point,
    order: u128,
    // cofactor: u128
}

impl Ecc {
    pub fn new(modulus: u128, a: u128, g: (u128, u128), order: u128) -> Self {
        let mont_space = MontgomerySpace::new(modulus);
        let a = mont_space.new_mont(&a);
        // let b = mont_space.new_mont(&b);
        let g = Point::new(mont_space.new_mont(&g.0), mont_space.new_mont(&g.1));
        Self {
            mont_space,
            // modulus,
            a,
            // b,
            g,
            order,
            // cofactor,
        }
    }

    pub fn double(&self, p: &Point) -> Point {
        let Self { mont_space: sp, .. } = self;
        if p.is_none {
            return Point::none_point(&self.mont_space);
        }

        //	lambda := NewFraction(3 * point.X * point.X , 2 * point.Y)
        let nom_lambda = sp.add(&sp.mul(&sp.mul(&p.x, &p.x), &sp.new_mont(&3)), &self.a);
        let den_lambda = sp.mul(&sp.new_mont(&2), &p.y);

        // 	fx := lambda.MulFrac(lambda).PlusInt(-2 * point.X)
        let den_fx = sp.mul(&den_lambda, &den_lambda);
        let nom_fx = sp.sub(
            &sp.mul(&nom_lambda, &nom_lambda),
            &sp.mul(&sp.mul(&den_fx, &p.x), &sp.new_mont(&2)),
        );

        // 	fy := lambda.MulFrac(fx.PlusInt(-1 * point1.X)).MulInt(-1).PlusInt(-1 * point1.Y)
        let mut nom_fy = sp.mul(
            &sp.neg(&nom_lambda),
            &sp.sub(&nom_fx, &sp.mul(&p.x, &den_fx)),
        );
        nom_fy = sp.sub(&nom_fy, &sp.mul(&sp.mul(&den_lambda, &den_fx), &p.y));
        let den_fy = sp.mul(&den_lambda, &den_fx);

        // let inverse_x = den_fx.try_inverse();
        let inverse_x = sp.inverse(&den_fx);
        if inverse_x.is_none() {
            return Point::none_point(sp);
        }
        let inverse_y = sp.inverse(&den_fy);
        if inverse_y.is_none() {
            return Point::none_point(sp);
        }

        let x = sp.mul(&nom_fx, &inverse_x.unwrap());
        let y = sp.mul(&nom_fy, &inverse_y.unwrap());

        Point::new(x, y)
    }

    pub fn add(&self, a: &Point, b: &Point) -> Point {
        if a.is_none {
            return b.clone();
        }
        if b.is_none {
            return a.clone();
        }

        if *a == *b {
            return self.double(a);
        }

        let Self { mont_space: sp, .. } = self;
        // 	lambda := NewFraction(point2.Y - point1.Y, point2.X - point1.X)
        let nom_lambda = sp.sub(&b.y, &a.y);
        let den_lambda = sp.sub(&b.x, &a.x);

        // 	fx := lambda.MulFrac(lambda).PlusInt(-1 * (point1.X + point2.X))
        let den_fx = sp.mul(&den_lambda, &den_lambda);
        let nom_fx = sp.sub(
            &sp.mul(&nom_lambda, &nom_lambda),
            &sp.mul(&den_fx, &sp.add(&a.x, &b.x)),
        );

        // 	fy := lambda.MulFrac(fx.PlusInt(-1 * point1.X)).MulInt(-1).PlusInt(-1 * point1.Y)
        let mut nom_fy = sp.mul(
            &sp.neg(&nom_lambda),
            &sp.sub(&nom_fx, &sp.mul(&a.x, &den_fx)),
        );
        nom_fy = sp.sub(&nom_fy, &sp.mul(&sp.mul(&den_lambda, &den_fx), &a.y));
        let den_fy = sp.mul(&den_lambda, &den_fx);

        let inverse_x = sp.inverse(&den_fx);
        if inverse_x.is_none() {
            return Point::none_point(sp);
        }
        let inverse_y = sp.inverse(&den_fy);
        if inverse_y.is_none() {
            return Point::none_point(sp);
        }

        let x = sp.mul(&nom_fx, &inverse_x.unwrap());
        let y = sp.mul(&nom_fy, &inverse_y.unwrap());

        Point::new(x, y)
    }

    pub fn mul(&self, a: &Point, mut k: u128) -> Point {
        if a.is_none {
            return Point::none_point(&self.mont_space);
        }
        k %= self.order;
        let mut b = a.clone();
        let mut res = Point::none_point(&self.mont_space);
        while k != 0 {
            if (k & 1) == 1 {
                res = self.add(&b, &res);
            }
            b = self.double(&b);
            k >>= 1;
        }
        res
    }

    pub fn new_point(&self, x: u128, y: u128) -> Point {
        let x = self.mont_space.new_mont(&x);
        let y = self.mont_space.new_mont(&y);
        Point::new(x, y)
    }

    pub fn norm_point(&self, p: &Point) -> (u128, u128) {
        p.to_norm(&self.mont_space)
    }

    pub fn get_public_key(&self, private_key: Vec<u8>) -> PublicKey {
        let key_num = u128::from_be_bytes(private_key.try_into().unwrap());
        let (x, y) = self.norm_point(&self.mul(&self.g, key_num));
        PublicKey::from_point(x, y)
    }

    pub fn generator(&self) -> Point {
        self.g.clone()
    }

    pub fn mont_space(&self) -> &MontgomerySpace {
        &self.mont_space
    }
}

#[cfg(test)]
mod tests {
    use crate::ecc::curve::Ecc;

    #[test]
    fn test_point() {
        let e = Ecc::new(0xFFFF_FFFF_0000_0001, 0, (0, 0), 0xFFFF_FFFF_0000_0001);
        let a = e.new_point(9, 27);
        let b = e.new_point(16, 64);
        let c = e.add(&a, &b);
        assert_eq!(c, e.new_point(1505856658727721172, 13122465168912998764));
        let d = e.mul(&a, 10);
        assert_eq!(d, e.new_point(13097188289284354868, 14997202928434057053));
    }
}
