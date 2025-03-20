pub struct Obj;


impl Obj {
    pub fn defaults<T: Default + PartialEq>(val: &T) -> bool {
        *val == T::default()
    }

 
    pub fn empty<T: PartialEq>(val: &Vec<T>) -> bool {
        val.len() == 0
    }

    
    
}