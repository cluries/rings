/// Any trait implementation for any type.

/// Any trait implementation for any type.
pub trait AnyTrait: std::any::Any {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub trait AnyTraitImplRef<T>: AnyTrait {
    fn downcast_ref(&self) -> Option<&T>;
}

pub trait AnyTraitImplMut<T>: AnyTrait {
    fn downcast_mut(&mut self) -> Option<&mut T>;
}

pub trait AnyTraitImpl<T>: AnyTraitImplRef<T> + AnyTraitImplMut<T> {}
