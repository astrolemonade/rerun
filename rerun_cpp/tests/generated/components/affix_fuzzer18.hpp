// DO NOT EDIT!: This file was auto-generated by crates/re_types_builder/src/codegen/cpp/mod.rs:54.
// Based on "crates/re_types/definitions/rerun/testing/components/fuzzy.fbs".

#pragma once

#include "../datatypes/affix_fuzzer4.hpp"

#include <cstdint>
#include <memory>
#include <optional>
#include <rerun/data_cell.hpp>
#include <rerun/result.hpp>
#include <utility>
#include <vector>

namespace arrow {
    class DataType;
    class ListBuilder;
    class MemoryPool;
} // namespace arrow

namespace rerun {
    namespace components {
        struct AffixFuzzer18 {
            std::optional<std::vector<rerun::datatypes::AffixFuzzer4>> many_optional_unions;

            /// Name of the component, used for serialization.
            static const char* NAME;

          public:
            AffixFuzzer18() = default;

            AffixFuzzer18(
                std::optional<std::vector<rerun::datatypes::AffixFuzzer4>> _many_optional_unions
            )
                : many_optional_unions(std::move(_many_optional_unions)) {}

            AffixFuzzer18& operator=(
                std::optional<std::vector<rerun::datatypes::AffixFuzzer4>> _many_optional_unions
            ) {
                many_optional_unions = std::move(_many_optional_unions);
                return *this;
            }

            /// Returns the arrow data type this type corresponds to.
            static const std::shared_ptr<arrow::DataType>& arrow_datatype();

            /// Creates a new array builder with an array of this type.
            static Result<std::shared_ptr<arrow::ListBuilder>> new_arrow_array_builder(
                arrow::MemoryPool* memory_pool
            );

            /// Fills an arrow array builder with an array of this type.
            static Error fill_arrow_array_builder(
                arrow::ListBuilder* builder, const AffixFuzzer18* elements, size_t num_elements
            );

            /// Creates a Rerun DataCell from an array of AffixFuzzer18 components.
            static Result<rerun::DataCell> to_data_cell(
                const AffixFuzzer18* instances, size_t num_instances
            );
        };
    } // namespace components
} // namespace rerun