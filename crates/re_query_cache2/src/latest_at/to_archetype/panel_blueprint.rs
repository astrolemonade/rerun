// DO NOT EDIT! This file was auto-generated by crates/re_types_builder/src/codegen/rust/to_archetype.rs

#![allow(unused_imports)]
#![allow(unused_parens)]
#![allow(clippy::clone_on_copy)]

use crate::CachedLatestAtResults;
use re_query2::{PromiseResolver, PromiseResult};
use re_types_core::{Archetype, Loggable as _};
use std::sync::Arc;

impl crate::ToArchetype<re_types_blueprint::blueprint::archetypes::PanelBlueprint>
    for CachedLatestAtResults
{
    #[inline]
    fn to_archetype(
        &self,
        resolver: &PromiseResolver,
    ) -> PromiseResult<crate::Result<re_types_blueprint::blueprint::archetypes::PanelBlueprint>>
    {
        re_tracing::profile_function!(
            <re_types_blueprint::blueprint::archetypes::PanelBlueprint>::name()
        );

        // --- Required ---

        // --- Recommended/Optional ---

        use re_types::blueprint::components::PanelExpanded;
        let expanded = if let Some(expanded) = self.get(<PanelExpanded>::name()) {
            match expanded.to_dense::<PanelExpanded>(resolver) {
                PromiseResult::Pending => return PromiseResult::Pending,
                PromiseResult::Error(promise_err) => return PromiseResult::Error(promise_err),
                PromiseResult::Ready(query_res) => match query_res {
                    Ok(data) => data.first().cloned(),
                    Err(query_err) => return PromiseResult::Ready(Err(query_err)),
                },
            }
        } else {
            None
        };

        // ---

        let arch = re_types_blueprint::blueprint::archetypes::PanelBlueprint { expanded };

        PromiseResult::Ready(Ok(arch))
    }
}
