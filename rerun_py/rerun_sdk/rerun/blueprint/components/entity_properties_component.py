# DO NOT EDIT! This file was auto-generated by crates/re_types_builder/src/codegen/python.rs
# Based on "crates/re_types/definitions/rerun/blueprint/components/entity_properties_component.fbs".

# You can extend this class by creating a "EntityPropertiesComponentExt" class in "entity_properties_component_ext.py".

from __future__ import annotations

from typing import Any, Sequence, Union

import numpy as np
import numpy.typing as npt
import pyarrow as pa
from attrs import define, field

from ..._baseclasses import BaseBatch, BaseExtensionType, ComponentBatchMixin
from ..._converters import (
    to_np_uint8,
)

__all__ = [
    "EntityPropertiesComponent",
    "EntityPropertiesComponentArrayLike",
    "EntityPropertiesComponentBatch",
    "EntityPropertiesComponentLike",
    "EntityPropertiesComponentType",
]


@define(init=False)
class EntityPropertiesComponent:
    """
    **Component**: The configurable set of overridable properties.

    Unstable. Used for the ongoing blueprint experimentations.
    """

    def __init__(self: Any, props: EntityPropertiesComponentLike):
        """Create a new instance of the EntityPropertiesComponent component."""

        # You can define your own __init__ function as a member of EntityPropertiesComponentExt in entity_properties_component_ext.py
        self.__attrs_init__(props=props)

    props: npt.NDArray[np.uint8] = field(converter=to_np_uint8)

    def __array__(self, dtype: npt.DTypeLike = None) -> npt.NDArray[Any]:
        # You can define your own __array__ function as a member of EntityPropertiesComponentExt in entity_properties_component_ext.py
        return np.asarray(self.props, dtype=dtype)


EntityPropertiesComponentLike = EntityPropertiesComponent
EntityPropertiesComponentArrayLike = Union[
    EntityPropertiesComponent,
    Sequence[EntityPropertiesComponentLike],
]


class EntityPropertiesComponentType(BaseExtensionType):
    _TYPE_NAME: str = "rerun.blueprint.components.EntityPropertiesComponent"

    def __init__(self) -> None:
        pa.ExtensionType.__init__(
            self, pa.list_(pa.field("item", pa.uint8(), nullable=False, metadata={})), self._TYPE_NAME
        )


class EntityPropertiesComponentBatch(BaseBatch[EntityPropertiesComponentArrayLike], ComponentBatchMixin):
    _ARROW_TYPE = EntityPropertiesComponentType()

    @staticmethod
    def _native_to_pa_array(data: EntityPropertiesComponentArrayLike, data_type: pa.DataType) -> pa.Array:
        raise NotImplementedError  # You need to implement native_to_pa_array_override in entity_properties_component_ext.py
