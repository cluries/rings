use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign, Index, IndexMut};
pub struct Object {

}

impl Object {
    pub fn new() -> Object {
        Object {}
    }
}

impl Add for Object {
    type Output = Object;

    fn add(self, other: Object) -> Object {
        other
    }
}

impl AddAssign for Object {
    fn add_assign(&mut self, _other: Object) {}
}

impl Sub for Object {
    type Output = Object;
    fn sub(self, other: Object) -> Object {
        other
    }
}

impl SubAssign for Object {
    fn sub_assign(&mut self, _other: Object) {}
}

impl Mul for Object {
    type Output = Object;
    fn mul(self, other: Object) -> Object {
        other
    }
}

impl MulAssign for Object {
    fn mul_assign(&mut self, _other: Object) {}
}

impl Div for Object {
    type Output = Object;
    fn div(self, other: Object) -> Object {
        other
    }
}

impl DivAssign for Object {
    fn div_assign(&mut self, _other: Object) {}
}

impl Index<&str> for Object {
    type Output = Object;
    fn index(&self, _index: &str) -> &Object {
        self
    }
}

impl IndexMut<&str> for Object {
    fn index_mut(&mut self, _index: &str) -> &mut Object {
        self
    }
}