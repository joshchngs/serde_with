//! Integration with [schemars v0.8](schemars_0_8).
//!
//! This module is only available if using the `schemars_0_8` feature of the crate.
//!
//! If you would like to add support for schemars to your own serde_with helpers
//! see [`JsonSchemaAs`].

use crate::{
    formats::{Flexible, Format, Separator, Strict},
    prelude::{Schema as WrapSchema, *},
};
use ::schemars_0_8::{
    gen::SchemaGenerator,
    schema::{
        ArrayValidation, InstanceType, Metadata, NumberValidation, Schema, SchemaObject,
        SubschemaValidation,
    },
    JsonSchema,
};
use std::borrow::Cow;

//===================================================================
// Trait Definition

/// A type which can be described as a JSON schema document.
///
/// This trait is as [`SerializeAs`] is to [`Serialize`] but for [`JsonSchema`].
/// You can use it to make your custom [`SerializeAs`] and [`DeserializeAs`]
/// types also support being described via JSON schemas.
///
/// It is used by the [`Schema`][1] type in order to implement [`JsonSchema`]
/// for the relevant types. [`Schema`][1] is used implicitly by the [`serde_as`]
/// macro to instruct `schemars` on how to generate JSON schemas for fields
/// annotated with `#[serde_as(as = "...")]` attributes.
///
/// # Examples
/// Suppose we have our very own `PositiveInt` type. Then we could add support
/// for generating a schema from it like this
///
/// ```
/// # extern crate schemars_0_8 as schemars;
/// # use serde::{Serialize, Serializer, Deserialize, Deserializer};
/// # use serde_with::{SerializeAs, DeserializeAs};
/// use serde_with::schemars_0_8::JsonSchemaAs;
/// use schemars::gen::SchemaGenerator;
/// use schemars::schema::Schema;
/// use schemars::JsonSchema;
///
/// struct PositiveInt;
///
/// impl SerializeAs<i32> for PositiveInt {
///     // ...
///     # fn serialize_as<S>(&value: &i32, ser: S) -> Result<S::Ok, S::Error>
///     # where
///     #    S: Serializer
///     # {
///     #    if value < 0 {
///     #        return Err(serde::ser::Error::custom(
///     #            "expected a positive integer value, got a negative one"
///     #        ));
///     #    }
///     #
///     #    value.serialize(ser)
///     # }
/// }
///
/// impl<'de> DeserializeAs<'de, i32> for PositiveInt {
///     // ...
///     # fn deserialize_as<D>(de: D) -> Result<i32, D::Error>
///     # where
///     #     D: Deserializer<'de>,
///     # {
///     #     match i32::deserialize(de) {
///     #         Ok(value) if value < 0 => Err(serde::de::Error::custom(
///     #             "expected a positive integer value, got a negative one"
///     #         )),
///     #         value => value
///     #     }
///     # }
/// }
///
/// impl JsonSchemaAs<i32> for PositiveInt {
///     fn schema_name() -> String {
///         "PositiveInt".into()
///     }
///
///     fn json_schema(gen: &mut SchemaGenerator) -> Schema {
///         let mut schema = <i32 as JsonSchema>::json_schema(gen).into_object();
///         schema.number().minimum = Some(0.0);
///         schema.into()
///     }
/// }
/// ```
///
/// [0]: crate::serde_as
/// [1]: crate::Schema
pub trait JsonSchemaAs<T: ?Sized> {
    /// Whether JSON Schemas generated for this type should be re-used where possible using the `$ref` keyword.
    ///
    /// For trivial types (such as primitives), this should return `false`. For more complex types, it should return `true`.
    /// For recursive types, this **must** return `true` to prevent infinite cycles when generating schemas.
    ///
    /// By default, this returns `true`.
    fn is_referenceable() -> bool {
        true
    }

    /// The name of the generated JSON Schema.
    ///
    /// This is used as the title for root schemas, and the key within the root's `definitions` property for subschemas.
    fn schema_name() -> String;

    /// Returns a string that uniquely identifies the schema produced by this type.
    ///
    /// This does not have to be a human-readable string, and the value will not itself be included in generated schemas.
    /// If two types produce different schemas, then they **must** have different `schema_id()`s,
    /// but two types that produce identical schemas should *ideally* have the same `schema_id()`.
    ///
    /// The default implementation returns the same value as `schema_name()`.
    fn schema_id() -> Cow<'static, str> {
        Cow::Owned(Self::schema_name())
    }

    /// Generates a JSON Schema for this type.
    ///
    /// If the returned schema depends on any [referenceable](JsonSchema::is_referenceable) schemas, then this method will
    /// add them to the [`SchemaGenerator`]'s schema definitions.
    ///
    /// This should not return a `$ref` schema.
    fn json_schema(gen: &mut SchemaGenerator) -> Schema;
}

impl<T, TA> JsonSchema for WrapSchema<T, TA>
where
    T: ?Sized,
    TA: JsonSchemaAs<T>,
{
    fn schema_name() -> String {
        TA::schema_name()
    }

    fn schema_id() -> Cow<'static, str> {
        TA::schema_id()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        TA::json_schema(gen)
    }

    fn is_referenceable() -> bool {
        TA::is_referenceable()
    }
}

//===================================================================
// Macro helpers

macro_rules! forward_schema {
    ($fwd:ty) => {
        fn schema_name() -> String {
            <$fwd as JsonSchema>::schema_name()
        }

        fn schema_id() -> Cow<'static, str> {
            <$fwd as JsonSchema>::schema_id()
        }

        fn json_schema(gen: &mut SchemaGenerator) -> Schema {
            <$fwd as JsonSchema>::json_schema(gen)
        }

        fn is_referenceable() -> bool {
            <$fwd as JsonSchema>::is_referenceable()
        }
    };
}

//===================================================================
// Common definitions for various std types

impl<'a, T: 'a, TA: 'a> JsonSchemaAs<&'a T> for &'a TA
where
    T: ?Sized,
    TA: JsonSchemaAs<T>,
{
    forward_schema!(&'a WrapSchema<T, TA>);
}

impl<'a, T: 'a, TA: 'a> JsonSchemaAs<&'a mut T> for &'a mut TA
where
    T: ?Sized,
    TA: JsonSchemaAs<T>,
{
    forward_schema!(&'a mut WrapSchema<T, TA>);
}

impl<T, TA> JsonSchemaAs<Option<T>> for Option<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(Option<WrapSchema<T, TA>>);
}

impl<T, TA> JsonSchemaAs<Box<T>> for Box<TA>
where
    T: ?Sized,
    TA: JsonSchemaAs<T>,
{
    forward_schema!(Box<WrapSchema<T, TA>>);
}

impl<T, TA> JsonSchemaAs<Rc<T>> for Rc<TA>
where
    T: ?Sized,
    TA: JsonSchemaAs<T>,
{
    forward_schema!(Rc<WrapSchema<T, TA>>);
}

impl<T, TA> JsonSchemaAs<Arc<T>> for Arc<TA>
where
    T: ?Sized,
    TA: JsonSchemaAs<T>,
{
    forward_schema!(Arc<WrapSchema<T, TA>>);
}

impl<T, TA> JsonSchemaAs<Vec<T>> for Vec<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(Vec<WrapSchema<T, TA>>);
}

impl<T, TA> JsonSchemaAs<VecDeque<T>> for VecDeque<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(VecDeque<WrapSchema<T, TA>>);
}

// schemars only requires that V implement JsonSchema for BTreeMap<K, V>
impl<K, V, KA, VA> JsonSchemaAs<BTreeMap<K, V>> for BTreeMap<KA, VA>
where
    VA: JsonSchemaAs<V>,
{
    forward_schema!(BTreeMap<WrapSchema<K, KA>, WrapSchema<V, VA>>);
}

// schemars only requires that V implement JsonSchema for HashMap<K, V>
impl<K, V, S, KA, VA> JsonSchemaAs<HashMap<K, V, S>> for HashMap<KA, VA, S>
where
    VA: JsonSchemaAs<V>,
{
    forward_schema!(HashMap<WrapSchema<K, KA>, WrapSchema<V, VA>, S>);
}

impl<T, TA> JsonSchemaAs<BTreeSet<T>> for BTreeSet<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(BTreeSet<WrapSchema<T, TA>>);
}

impl<T, TA, S> JsonSchemaAs<T> for HashSet<TA, S>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(HashSet<WrapSchema<T, TA>, S>);
}

impl<T, TA, const N: usize> JsonSchemaAs<[T; N]> for [TA; N]
where
    TA: JsonSchemaAs<T>,
{
    fn schema_name() -> String {
        std::format!("[{}; {}]", <WrapSchema<T, TA>>::schema_name(), N)
    }

    fn schema_id() -> Cow<'static, str> {
        std::format!("[{}; {}]", <WrapSchema<T, TA>>::schema_id(), N).into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let (max, min) = match N.try_into() {
            Ok(len) => (Some(len), Some(len)),
            Err(_) => (None, Some(u32::MAX)),
        };

        SchemaObject {
            instance_type: Some(InstanceType::Array.into()),
            array: Some(Box::new(ArrayValidation {
                items: Some(gen.subschema_for::<WrapSchema<T, TA>>().into()),
                max_items: max,
                min_items: min,
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }

    fn is_referenceable() -> bool {
        false
    }
}

macro_rules! schema_for_tuple {
    (
        ( $( $ts:ident )+ )
        ( $( $as:ident )+ )
    ) => {
        impl<$($ts,)+ $($as,)+> JsonSchemaAs<($($ts,)+)> for ($($as,)+)
        where
            $( $as: JsonSchemaAs<$ts>, )+
        {
            forward_schema!(( $( WrapSchema<$ts, $as>, )+ ));
        }
    }
}

impl JsonSchemaAs<()> for () {
    forward_schema!(());
}

// schemars only implements JsonSchema for tuples up to 15 elements so we do
// the same here.
schema_for_tuple!((T0)(A0));
schema_for_tuple!((T0 T1) (A0 A1));
schema_for_tuple!((T0 T1 T2) (A0 A1 A2));
schema_for_tuple!((T0 T1 T2 T3) (A0 A1 A2 A3));
schema_for_tuple!((T0 T1 T2 T3 T4) (A0 A1 A2 A3 A4));
schema_for_tuple!((T0 T1 T2 T3 T4 T5) (A0 A1 A2 A3 A4 A5));
schema_for_tuple!((T0 T1 T2 T3 T4 T5 T6) (A0 A1 A2 A3 A4 A5 A6));
schema_for_tuple!((T0 T1 T2 T3 T4 T5 T6 T7) (A0 A1 A2 A3 A4 A5 A6 A7));
schema_for_tuple!((T0 T1 T2 T3 T4 T5 T6 T7 T8) (A0 A1 A2 A3 A4 A5 A6 A7 A8));
schema_for_tuple!((T0 T1 T2 T3 T4 T5 T6 T7 T8 T9) (A0 A1 A2 A3 A4 A5 A6 A7 A8 A9));
schema_for_tuple!((T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10) (A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10));
schema_for_tuple!((T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11) (A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11));
schema_for_tuple!(
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12)
    (A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12)
);
schema_for_tuple!(
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13)
    (A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13)
);
schema_for_tuple!(
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14)
    (A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14)
);
schema_for_tuple!(
    (T0 T1 T2 T3 T4 T5 T6 T7 T8 T9 T10 T11 T12 T13 T14 T15)
    (A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15)
);

//===================================================================
// Impls for serde_with types.

impl<T: JsonSchema> JsonSchemaAs<T> for Same {
    forward_schema!(T);
}

impl<T> JsonSchemaAs<T> for DisplayFromStr {
    forward_schema!(String);
}

impl JsonSchemaAs<bool> for BoolFromInt<Strict> {
    fn schema_name() -> String {
        "BoolFromInt<Strict>".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "serde_with::BoolFromInt<Strict>".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Integer.into()),
            number: Some(Box::new(NumberValidation {
                minimum: Some(0.0),
                maximum: Some(1.0),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }

    fn is_referenceable() -> bool {
        false
    }
}

impl JsonSchemaAs<bool> for BoolFromInt<Flexible> {
    fn schema_name() -> String {
        "BoolFromInt<Flexible>".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "serde_with::BoolFromInt<Flexible>".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            instance_type: Some(InstanceType::Integer.into()),
            ..Default::default()
        }
        .into()
    }

    fn is_referenceable() -> bool {
        false
    }
}

impl<'a, T: 'a> JsonSchemaAs<Cow<'a, T>> for BorrowCow
where
    T: ?Sized + ToOwned,
    Cow<'a, T>: JsonSchema,
{
    forward_schema!(Cow<'a, T>);
}

impl<T> JsonSchemaAs<T> for Bytes {
    forward_schema!(Vec<u8>);
}

impl JsonSchemaAs<Vec<u8>> for BytesOrString {
    fn schema_name() -> String {
        "BytesOrString".into()
    }

    fn schema_id() -> Cow<'static, str> {
        "serde_with::BytesOrString".into()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(std::vec![
                    gen.subschema_for::<Vec<u8>>(),
                    SchemaObject {
                        instance_type: Some(InstanceType::String.into()),
                        metadata: Some(Box::new(Metadata {
                            write_only: true,
                            ..Default::default()
                        })),
                        ..Default::default()
                    }
                    .into()
                ]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }

    fn is_referenceable() -> bool {
        false
    }
}

impl<T, TA> JsonSchemaAs<T> for DefaultOnError<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(WrapSchema<T, TA>);
}

impl<T, TA> JsonSchemaAs<T> for DefaultOnNull<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(Option<WrapSchema<T, TA>>);
}

impl<O, T: JsonSchema> JsonSchemaAs<O> for FromInto<T> {
    forward_schema!(T);
}

impl<O, T: JsonSchema> JsonSchemaAs<O> for FromIntoRef<T> {
    forward_schema!(T);
}

impl<T, U: JsonSchema> JsonSchemaAs<T> for TryFromInto<U> {
    forward_schema!(U);
}

impl<T, U: JsonSchema> JsonSchemaAs<T> for TryFromIntoRef<U> {
    forward_schema!(U);
}

macro_rules! schema_for_map {
    ($type:ty) => {
        impl<K, V, KA, VA> JsonSchemaAs<$type> for Map<KA, VA>
        where
            VA: JsonSchemaAs<V>,
        {
            forward_schema!(WrapSchema<BTreeMap<K, V>, BTreeMap<KA, VA>>);
        }
    };
}

schema_for_map!([(K, V)]);
schema_for_map!(BTreeSet<(K, V)>);
schema_for_map!(BinaryHeap<(K, V)>);
schema_for_map!(Box<[(K, V)]>);
schema_for_map!(LinkedList<(K, V)>);
schema_for_map!(Vec<(K, V)>);
schema_for_map!(VecDeque<(K, V)>);

impl<K, V, S, KA, VA> JsonSchemaAs<HashSet<(K, V), S>> for Map<KA, VA>
where
    VA: JsonSchemaAs<V>,
{
    forward_schema!(WrapSchema<BTreeMap<K, V>, BTreeMap<KA, VA>>);
}

impl<K, V, KA, VA, const N: usize> JsonSchemaAs<[(K, V); N]> for Map<KA, VA>
where
    VA: JsonSchemaAs<V>,
{
    forward_schema!(WrapSchema<BTreeMap<K, V>, BTreeMap<KA, VA>>);
}

macro_rules! map_first_last_wins_schema {
    ($(=> $extra:ident)? $type:ty) => {
        impl<K, V, $($extra,)? KA, VA> JsonSchemaAs<$type> for MapFirstKeyWins<KA, VA>
        where
            VA: JsonSchemaAs<V>,
        {
            forward_schema!(BTreeMap<WrapSchema<K, KA>, WrapSchema<V, VA>>);
        }

        impl<K, V, $($extra,)? KA, VA> JsonSchemaAs<$type> for MapPreventDuplicates<KA, VA>
        where
            VA: JsonSchemaAs<V>,
        {
            forward_schema!(BTreeMap<WrapSchema<K, KA>, WrapSchema<V, VA>>);
        }
    }
}

map_first_last_wins_schema!(BTreeMap<K, V>);
map_first_last_wins_schema!(=> S HashMap<K, V, S>);
#[cfg(feature = "hashbrown_0_14")]
map_first_last_wins_schema!(=> S hashbrown_0_14::HashMap<K, V, S>);
#[cfg(feature = "indexmap_1")]
map_first_last_wins_schema!(=> S indexmap_1::IndexMap<K, V, S>);
#[cfg(feature = "indexmap_2")]
map_first_last_wins_schema!(=> S indexmap_2::IndexMap<K, V, S>);

impl<T, TA> JsonSchemaAs<T> for SetLastValueWins<TA>
where
    TA: JsonSchemaAs<T>,
{
    fn schema_id() -> Cow<'static, str> {
        std::format!(
            "serde_with::SetLastValueWins<{}>",
            <WrapSchema<T, TA> as JsonSchema>::schema_id()
        )
        .into()
    }

    fn schema_name() -> String {
        std::format!(
            "SetLastValueWins<{}>",
            <WrapSchema<T, TA> as JsonSchema>::schema_name()
        )
    }

    fn json_schema(gen: &mut ::schemars_0_8::gen::SchemaGenerator) -> Schema {
        let schema = <WrapSchema<T, TA> as JsonSchema>::json_schema(gen);
        let mut schema = schema.into_object();

        // We explicitly allow duplicate items since the whole point of
        // SetLastValueWins is to take the duplicate value.
        if let Some(array) = &mut schema.array {
            array.unique_items = None;
        }

        schema.into()
    }

    fn is_referenceable() -> bool {
        false
    }
}

impl<T, TA> JsonSchemaAs<T> for SetPreventDuplicates<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(WrapSchema<T, TA>);
}

impl<SEP, T, TA> JsonSchemaAs<T> for StringWithSeparator<SEP, TA>
where
    SEP: Separator,
{
    forward_schema!(String);
}

impl<T, TA> JsonSchemaAs<Vec<T>> for VecSkipError<TA>
where
    TA: JsonSchemaAs<T>,
{
    forward_schema!(Vec<WrapSchema<T, TA>>);
}

mod timespan {
    use super::*;

    /// Internal helper trait used to constrain which types we implement
    /// `JsonSchemaAs<T>` for.
    pub trait TimespanSchemaTarget<F> {
        /// Whether F is signed.
        const SIGNED: bool = true;

        /// Whether F is String
        const STRING: bool;
    }

    macro_rules! is_string {
        (String) => {
            true
        };
        ($name:ty) => {
            false
        };
    }

    macro_rules! declare_timespan_target {
        ( $target:ty { $($format:ty),* $(,)? } )=> {
            $(
                impl TimespanSchemaTarget<$format> for $target {
                    const STRING: bool = is_string!($format);
                }
            )*
        }
    }

    impl TimespanSchemaTarget<u64> for Duration {
        const SIGNED: bool = false;
        const STRING: bool = false;
    }

    impl TimespanSchemaTarget<f64> for Duration {
        const SIGNED: bool = false;
        const STRING: bool = false;
    }

    impl TimespanSchemaTarget<String> for Duration {
        const SIGNED: bool = false;
        const STRING: bool = true;
    }

    declare_timespan_target!(SystemTime { i64, f64, String });

    #[cfg(feature = "chrono_0_4")]
    declare_timespan_target!(::chrono_0_4::Duration { i64, f64, String });
    #[cfg(feature = "chrono_0_4")]
    declare_timespan_target!(::chrono_0_4::DateTime<::chrono_0_4::Utc> { i64, f64, String });
    #[cfg(feature = "chrono_0_4")]
    declare_timespan_target!(::chrono_0_4::DateTime<::chrono_0_4::Local> { i64, f64, String });
    #[cfg(feature = "chrono_0_4")]
    declare_timespan_target!(::chrono_0_4::NaiveDateTime { i64, f64, String });

    #[cfg(feature = "time_0_3")]
    declare_timespan_target!(::time_0_3::Duration { i64, f64, String });
    #[cfg(feature = "time_0_3")]
    declare_timespan_target!(::time_0_3::OffsetDateTime { i64, f64, String });
    #[cfg(feature = "time_0_3")]
    declare_timespan_target!(::time_0_3::PrimitiveDateTime { i64, f64, String });
}

use self::timespan::TimespanSchemaTarget;

/// Internal type used for the base impls on DurationXXX and TimestampYYY types.
///
/// This allows the JsonSchema impls that are Strict to be generic without
/// committing to it as part of the public API.
struct Timespan<Format, Strictness>(PhantomData<(Format, Strictness)>);

impl<T, F> JsonSchemaAs<T> for Timespan<F, Strict>
where
    T: TimespanSchemaTarget<F>,
    F: Format + JsonSchema,
{
    forward_schema!(F);
}

fn flexible_timespan_schema(signed: bool, is_string: bool) -> Schema {
    let mut number = SchemaObject {
        instance_type: Some(InstanceType::Number.into()),
        number: (!signed).then(|| {
            Box::new(NumberValidation {
                minimum: Some(0.0),
                ..Default::default()
            })
        }),
        ..Default::default()
    };

    let mut string = SchemaObject {
        instance_type: Some(InstanceType::String.into()),
        ..Default::default()
    };

    if is_string {
        number.metadata().write_only = true;
    } else {
        string.metadata().write_only = true;
    }

    SchemaObject {
        subschemas: Some(Box::new(SubschemaValidation {
            one_of: Some(std::vec![number.into(), string.into()]),
            ..Default::default()
        })),
        ..Default::default()
    }
    .into()
}

impl<T, F> JsonSchemaAs<T> for Timespan<F, Flexible>
where
    T: TimespanSchemaTarget<F>,
    F: Format + JsonSchema,
{
    fn schema_name() -> String {
        match <T as TimespanSchemaTarget<F>>::STRING {
            true => "FlexibleStringTimespan".into(),
            false => "FlexibleTimespan".into(),
        }
    }

    fn schema_id() -> Cow<'static, str> {
        match <T as TimespanSchemaTarget<F>>::STRING {
            true => "serde_with::FlexibleStringTimespan".into(),
            false => "serde_with::FlexibleTimespan".into(),
        }
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        flexible_timespan_schema(
            <T as TimespanSchemaTarget<F>>::SIGNED,
            <T as TimespanSchemaTarget<F>>::STRING,
        )
    }

    fn is_referenceable() -> bool {
        false
    }
}

macro_rules! forward_duration_schema {
    ($ty:ident) => {
        impl<T, F> JsonSchemaAs<T> for $ty<F, Strict>
        where
            T: TimespanSchemaTarget<F>,
            F: Format + JsonSchema
        {
            forward_schema!(WrapSchema<T, Timespan<F, Strict>>);
        }

        impl<T, F> JsonSchemaAs<T> for $ty<F, Flexible>
        where
            T: TimespanSchemaTarget<F>,
            F: Format + JsonSchema
        {
            forward_schema!(WrapSchema<T, Timespan<F, Flexible>>);
        }
    };
}

forward_duration_schema!(DurationSeconds);
forward_duration_schema!(DurationMilliSeconds);
forward_duration_schema!(DurationMicroSeconds);
forward_duration_schema!(DurationNanoSeconds);

forward_duration_schema!(DurationSecondsWithFrac);
forward_duration_schema!(DurationMilliSecondsWithFrac);
forward_duration_schema!(DurationMicroSecondsWithFrac);
forward_duration_schema!(DurationNanoSecondsWithFrac);

forward_duration_schema!(TimestampSeconds);
forward_duration_schema!(TimestampMilliSeconds);
forward_duration_schema!(TimestampMicroSeconds);
forward_duration_schema!(TimestampNanoSeconds);

forward_duration_schema!(TimestampSecondsWithFrac);
forward_duration_schema!(TimestampMilliSecondsWithFrac);
forward_duration_schema!(TimestampMicroSecondsWithFrac);
forward_duration_schema!(TimestampNanoSecondsWithFrac);
