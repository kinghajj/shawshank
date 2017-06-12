/// Facilitates defining tuple structs that may be used as custom IDs.
///
/// See the [section from `ArenaSet`][ex] for an example.
///
/// [ex]: struct.ArenaSet.html#custom-id-types
#[macro_export]
macro_rules! custom_intern_id {
    ( $name:ident, $base:ty, $min:expr, $max:expr ) => {
        #[derive(Clone, Copy, Eq, PartialEq, Debug)]
        struct $name($base);

        impl ::num_traits::Bounded for $name {
            fn min_value() -> Self {
                $name($min)
            }
            fn max_value() -> Self {
                $name($max)
            }
        }

        impl ::num_traits::ToPrimitive for $name {
            fn to_i64(&self) -> Option<i64> { self.0.to_i64() }
            fn to_u64(&self) -> Option<u64> { self.0.to_u64() }
        }

        impl ::num_traits::FromPrimitive for $name {
            fn from_i64(n: i64) -> Option<Self> { <$base as ::num_traits::FromPrimitive>::from_i64(n).map(|x| $name(x)) }
            fn from_u64(n: u64) -> Option<Self> { <$base as ::num_traits::FromPrimitive>::from_u64(n).map(|x| $name(x)) }
        }
    };
    ( $name:ident, $base:ty ) => {
        custom_intern_id!($name, $base, <$base as ::num::Bounded>::min_value(), <$base as ::num::Bounded>::max_value());
    };
}
