use std::{collections::BTreeMap, marker::PhantomData};

use arrow2::array::{Array, PrimitiveArray};
use re_format::arrow;
use re_log_types::{DataCell, RowId};
use re_types::{
    components::InstanceKey, Archetype, Component, ComponentName, DeserializationError,
    DeserializationResult, Loggable,
};

use crate::QueryError;

/// A type-erased array of [`Component`] values and the corresponding [`InstanceKey`] keys.
///
/// See: [`crate::get_component_with_instances`]
#[derive(Clone, Debug)]
pub struct ComponentWithInstances {
    pub(crate) instance_keys: DataCell,
    pub(crate) values: DataCell,
}

impl ComponentWithInstances {
    /// Name of the [`Component`]
    #[inline]
    pub fn name(&self) -> ComponentName {
        self.values.component_name()
    }

    /// Number of values. 1 for splats.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.num_instances() as _
    }

    #[inline]
    /// Whether this [`ComponentWithInstances`] contains any data
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Iterate over the [`InstanceKey`]s.
    #[inline]
    pub fn iter_instance_keys(
        &self,
    ) -> impl Iterator<Item = re_types::components::InstanceKey> + '_ {
        self.instance_keys.to_native::<InstanceKey>()
    }

    /// Iterate over the values and convert them to a native [`Component`]
    #[inline]
    pub fn iter_values<'a, C: Component + 'a>(
        &'a self,
    ) -> crate::Result<impl Iterator<Item = Option<C>> + 'a> {
        if C::name() != self.name() {
            return Err(QueryError::TypeMismatch {
                actual: self.name(),
                requested: C::name(),
            });
        }

        Ok(self.values.try_to_native_opt::<'a, C>()?)
    }

    /// Look up the value that corresponds to a given [`InstanceKey`] and convert to [`Component`]
    pub fn lookup<C: Component>(&self, instance_key: &InstanceKey) -> crate::Result<C> {
        if C::name() != self.name() {
            return Err(QueryError::TypeMismatch {
                actual: self.name(),
                requested: C::name(),
            });
        }
        let arr = self
            .lookup_arrow(instance_key)
            .map_or_else(|| Err(QueryError::ComponentNotFound), Ok)?;

        let mut iter = C::try_from_arrow(arr.as_ref())?.into_iter();

        let val = iter
            .next()
            .map_or_else(|| Err(QueryError::ComponentNotFound), Ok)?;
        Ok(val)
    }

    /// Look up the value that corresponds to a given [`InstanceKey`] and return as an arrow [`Array`]
    pub fn lookup_arrow(&self, instance_key: &InstanceKey) -> Option<Box<dyn Array>> {
        let keys = self
            .instance_keys
            .as_arrow_ref()
            .as_any()
            .downcast_ref::<PrimitiveArray<u64>>()?
            .values();

        // If the value is splatted, return the offset of the splat
        let offset = if keys.len() == 1 && keys[0] == InstanceKey::SPLAT.0 {
            0
        } else {
            // Otherwise binary search to find the offset of the instance
            keys.binary_search(&instance_key.0).ok()?
        };

        (self.len() > offset)
            .then(|| self.values.as_arrow_ref().sliced(offset, 1))
            .or_else(|| {
                re_log::error_once!("found corrupt cell -- mismatched number of instances");
                None
            })
    }

    /// Produce a [`ComponentWithInstances`] from native [`Component`] types
    pub fn from_native<'a, C: Component + Clone + 'a>(
        instance_keys: impl IntoIterator<Item = impl Into<::std::borrow::Cow<'a, InstanceKey>>>,
        values: impl IntoIterator<Item = impl Into<::std::borrow::Cow<'a, C>>>,
    ) -> ComponentWithInstances {
        let instance_keys = InstanceKey::to_arrow(instance_keys, None);
        let values = C::to_arrow(values, None);
        ComponentWithInstances {
            instance_keys: DataCell::from_arrow(InstanceKey::name(), instance_keys),
            values: DataCell::from_arrow(C::name(), values),
        }
    }
}

/// Iterator over a single [`Component`] joined onto a primary [`Component`]
///
/// This is equivalent to a left join between one table made up of the
/// [`InstanceKey`]s from the primary component and another table with the
/// [`InstanceKey`]s and values of the iterated [`Component`].
///
/// Instances have a [`InstanceKey::SPLAT`] key that will cause the value to be
/// repeated for the entirety of the join.
///
/// For example
/// ```text
/// primary
/// +----------+
/// | instance |
/// +----------+
/// | key0     |
/// | key1     |
/// | Key2     |
///
/// component
/// +----------+-------+
/// | instance | value |
/// +----------+-------+
/// | key0     | val0  |
/// | Key2     | val2  |
///
/// SELECT value FROM LEFT JOIN primary.instance = component.instance;
///
/// output
/// +-------+
/// | value |
/// +-------+
/// | val0  |
/// | NULL  |
/// | val2  |
///
/// ```
pub struct ComponentJoinedIterator<IIter1, IIter2, VIter, Val> {
    pub primary_instance_key_iter: IIter1,
    pub component_instance_key_iter: IIter2,
    pub component_value_iter: VIter,
    pub next_component_instance_key: Option<InstanceKey>,
    pub splatted_component_value: Option<Val>,
}

impl<IIter1, IIter2, VIter, C> Iterator for ComponentJoinedIterator<IIter1, IIter2, VIter, C>
where
    IIter1: Iterator<Item = InstanceKey>,
    IIter2: Iterator<Item = InstanceKey>,
    VIter: Iterator<Item = Option<C>>,
    C: Clone,
{
    type Item = Option<C>;

    fn next(&mut self) -> Option<Option<C>> {
        // For each value of primary_instance_iter we must find a result
        if let Some(primary_key) = self.primary_instance_key_iter.next() {
            loop {
                match &self.next_component_instance_key {
                    // If we have a next component key, we either...
                    Some(instance_key) => {
                        if instance_key.is_splat() {
                            if self.splatted_component_value.is_none() {
                                self.splatted_component_value =
                                    self.component_value_iter.next().flatten();
                            }
                            break Some(self.splatted_component_value.clone());
                        } else {
                            match primary_key.0.cmp(&instance_key.0) {
                                // Return a None if the primary_key hasn't reached it yet
                                std::cmp::Ordering::Less => break Some(None),
                                // Return the value if the keys match
                                std::cmp::Ordering::Equal => {
                                    self.next_component_instance_key =
                                        self.component_instance_key_iter.next();
                                    break self.component_value_iter.next();
                                }
                                // Skip this component if the key is behind the primary key
                                std::cmp::Ordering::Greater => {
                                    _ = self.component_value_iter.next();
                                    self.next_component_instance_key =
                                        self.component_instance_key_iter.next();
                                }
                            }
                        }
                    }
                    // Otherwise, we ran out of component elements. Just return
                    // None until the primary iter ends.
                    None => break Some(None),
                };
            }
        } else {
            None
        }
    }
}

/// A view of an [`Archetype`] at a particular point in time returned by [`crate::get_component_with_instances`]
///
/// The required [`Component`]s of an [`ArchetypeView`] determines the length of an entity
/// batch. When iterating over individual components, they will be implicitly joined onto
/// the required [`Component`]s using [`InstanceKey`] values.
#[derive(Clone, Debug)]
pub struct ArchetypeView<A: Archetype> {
    pub(crate) row_id: RowId,
    pub(crate) components: BTreeMap<ComponentName, ComponentWithInstances>,
    pub(crate) phantom: PhantomData<A>,
}

impl<A: Archetype> std::fmt::Display for ArchetypeView<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let first_required = self.required_comp();

        let primary_table = arrow::format_table(
            [
                first_required.instance_keys.as_arrow_ref(),
                first_required.values.as_arrow_ref(),
            ],
            ["InstanceId", first_required.name().as_ref()],
        );

        f.write_fmt(format_args!("ArchetypeView:\n{primary_table}"))
    }
}

impl<A: Archetype> ArchetypeView<A> {
    #[inline]
    pub fn num_instances(&self) -> usize {
        self.required_comp().len()
    }

    #[inline]
    pub fn row_id(&self) -> RowId {
        self.row_id
    }
}

impl<A: Archetype> ArchetypeView<A> {
    #[inline]
    fn required_comp(&self) -> &ComponentWithInstances {
        // TODO(jleibs): Do all archetypes always have at least 1 required components?
        let first_required = A::required_components()[0];
        &self.components[&first_required]
    }

    /// Iterate over the [`InstanceKey`]s.
    #[inline]
    pub fn iter_instance_keys(&self) -> impl Iterator<Item = InstanceKey> + '_ {
        // TODO(https://github.com/rerun-io/rerun/issues/2750): Maybe make this an intersection instead
        self.required_comp().iter_instance_keys()
    }

    /// Check if the entity has a component and its not empty
    #[inline]
    pub fn has_component<C: Component>(&self) -> bool {
        let name = C::name();
        self.components.get(&name).map_or(false, |c| !c.is_empty())
    }

    /// Iterate over the values of a required [`Component`].
    #[inline]
    pub fn iter_required_component<'a, C: Component + Default + 'a>(
        &'a self,
    ) -> DeserializationResult<impl Iterator<Item = C> + '_> {
        re_tracing::profile_function!(C::name().as_str());
        debug_assert!(A::required_components()
            .iter()
            .any(|c| c.as_ref() == C::name()));
        let component = self.components.get(&C::name());

        if let Some(component) = component {
            let component_value_iter = component
                .values
                .try_to_native()
                .map_err(|err| DeserializationError::DataCellError(err.to_string()))?;

            Ok(component_value_iter)
        } else {
            Err(re_types::DeserializationError::MissingData {
                backtrace: ::backtrace::Backtrace::new_unresolved(),
            })
        }
    }

    /// Iterate over the values of an optional `Component`.
    ///
    /// Always produces an iterator that matches the length of a primary
    /// component by joining on the `InstanceKey` values.
    #[inline]
    pub fn iter_optional_component<'a, C: Component + Clone + 'a>(
        &'a self,
    ) -> DeserializationResult<impl Iterator<Item = Option<C>> + '_> {
        let component = self.components.get(&C::name());

        if let Some(component) = component {
            let primary_instance_key_iter = self.iter_instance_keys();

            let mut component_instance_key_iter = component.iter_instance_keys();

            let component_value_iter =
                C::try_from_arrow_opt(component.values.as_arrow_ref())?.into_iter();

            let next_component_instance_key = component_instance_key_iter.next();

            Ok(itertools::Either::Left(ComponentJoinedIterator {
                primary_instance_key_iter,
                component_instance_key_iter,
                component_value_iter,
                next_component_instance_key,
                splatted_component_value: None,
            }))
        } else {
            let primary = self.required_comp();
            let nulls = (0..primary.len()).map(|_| None);
            Ok(itertools::Either::Right(nulls))
        }
    }

    /// Helper function to produce an [`ArchetypeView`] from a collection of [`ComponentWithInstances`]
    #[inline]
    pub fn from_components(
        row_id: RowId,
        components: impl IntoIterator<Item = ComponentWithInstances>,
    ) -> Self {
        Self {
            row_id,
            components: components
                .into_iter()
                .map(|comp| (comp.name(), comp))
                .collect(),
            phantom: PhantomData,
        }
    }
}

#[test]
fn lookup_value() {
    use re_types::components::{InstanceKey, Point2D};

    let instance_keys = InstanceKey::from_iter(0..5);

    let points = [
        Point2D::new(1.0, 2.0), //
        Point2D::new(3.0, 4.0),
        Point2D::new(5.0, 6.0),
        Point2D::new(7.0, 8.0),
        Point2D::new(9.0, 10.0),
    ];

    let component = ComponentWithInstances::from_native(instance_keys, points);

    let missing_value = component.lookup_arrow(&InstanceKey(5));
    assert_eq!(missing_value, None);

    let value = component.lookup_arrow(&InstanceKey(2)).unwrap();

    let expected_point = [points[2]];
    let expected_arrow = Point2D::to_arrow(expected_point, None);

    assert_eq!(expected_arrow, value);

    let instance_keys = [
        InstanceKey(17),
        InstanceKey(47),
        InstanceKey(48),
        InstanceKey(99),
        InstanceKey(472),
    ];

    let component = ComponentWithInstances::from_native(instance_keys, points);

    let missing_value = component.lookup_arrow(&InstanceKey(46));
    assert_eq!(missing_value, None);

    let value = component.lookup_arrow(&InstanceKey(99)).unwrap();

    let expected_point = [points[3]];
    let expected_arrow = Point2D::to_arrow(expected_point, None);

    assert_eq!(expected_arrow, value);

    // Lookups with serialization

    let value = component.lookup::<Point2D>(&InstanceKey(99)).unwrap();
    assert_eq!(expected_point[0], value);

    let missing_value = component.lookup::<Point2D>(&InstanceKey(46));
    assert!(matches!(
        missing_value.err().unwrap(),
        QueryError::ComponentNotFound
    ));

    let missing_value = component.lookup::<re_components::Rect2D>(&InstanceKey(99));
    assert!(matches!(
        missing_value.err().unwrap(),
        QueryError::TypeMismatch { .. }
    ));
}

#[test]
fn lookup_splat() {
    use re_types::components::{InstanceKey, Point2D};
    let instances = [
        InstanceKey::SPLAT, //
    ];
    let points = [
        Point2D::new(1.0, 2.0), //
    ];

    let component = ComponentWithInstances::from_native(instances, points);

    // Any instance we look up will return the slatted value
    let value = component.lookup::<Point2D>(&InstanceKey(1)).unwrap();
    assert_eq!(points[0], value);

    let value = component.lookup::<Point2D>(&InstanceKey(99)).unwrap();
    assert_eq!(points[0], value);
}
