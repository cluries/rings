/// Any trait implementation for any type.

/// Any trait implementation for any type.
///
/// Defaults implementaion
///
/// impl AnyTrait for XXXX {
///     fn as_any(&self) -> &dyn std::any::Any {
///         self
///     }
///
///     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
///         self
///     }
/// }
///
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

// impl<T: std::any::Any + Send + Sync> AnyTrait for T {
//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }
//
//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
//         self
//     }
// }
