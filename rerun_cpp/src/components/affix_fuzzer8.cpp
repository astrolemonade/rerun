// NOTE: This file was autogenerated by re_types_builder; DO NOT EDIT.
// Based on "crates/re_types/definitions/rerun/testing/components/fuzzy.fbs"

#include "affix_fuzzer8.hpp"

#include <arrow/api.h>

namespace rr {
    namespace components {
        std::shared_ptr<arrow::DataType> AffixFuzzer8::to_arrow_datatype() {
            return arrow::float32();
        }

        arrow::Result<std::shared_ptr<arrow::FloatBuilder>> AffixFuzzer8::new_arrow_array_builder(
            arrow::MemoryPool* memory_pool
        ) {
            if (!memory_pool) {
                return arrow::Status::Invalid("Memory pool is null.");
            }

            return arrow::Result(std::make_shared<arrow::FloatBuilder>(memory_pool));
        }

        arrow::Status AffixFuzzer8::fill_arrow_array_builder(
            arrow::FloatBuilder* builder, const AffixFuzzer8* elements, size_t num_elements
        ) {
            if (!builder) {
                return arrow::Status::Invalid("Passed array builder is null.");
            }
            if (!elements) {
                return arrow::Status::Invalid("Cannot serialize null pointer to arrow array.");
            }

            ARROW_RETURN_NOT_OK(builder->Reserve(num_elements));
            for (auto elem_idx = 0; elem_idx < num_elements; elem_idx += 1) {
                const auto& element = elements[elem_idx];
                if (element.single_float_optional.has_value()) {
                    ARROW_RETURN_NOT_OK(builder->Append(element.single_float_optional.value()));
                } else {
                    ARROW_RETURN_NOT_OK(builder->AppendNull());
                }
            }

            return arrow::Status::OK();
        }
    } // namespace components
} // namespace rr