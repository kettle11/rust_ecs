use super::{sparse_set, ComponentId};

/// [ArchetypeLookup] is used to efficiently find [Archetype]s that match [Filter]s
pub(crate) struct ArchetypeLookup {
    // The value stored in the SparseSet is the index within the Archetype
    exact_component_ids_to_archetype: std::collections::HashMap<Vec<ComponentId>, usize>,
    component_id_to_archetypes:
        std::collections::HashMap<ComponentId, sparse_set::SparseSet<usize>>,
    total_archetype_count: usize,
}

impl ArchetypeLookup {
    pub(crate) fn new() -> Self {
        Self {
            component_id_to_archetypes: std::collections::HashMap::new(),
            exact_component_ids_to_archetype: std::collections::HashMap::new(),
            total_archetype_count: 0,
        }
    }

    /// [ComponentId]s passed in must be sorted according to how they're stored in the [Archetype].
    pub(crate) fn new_archetype(&mut self, component_ids: &[ComponentId]) {
        let archetype_index = self.total_archetype_count;
        self.total_archetype_count += 1;
        for (index_within_archetype, &component_id) in component_ids.iter().enumerate() {
            let component_id_to_archetypes = self
                .component_id_to_archetypes
                .entry(component_id)
                .or_insert_with(|| sparse_set::SparseSet::new());
            component_id_to_archetypes.insert(archetype_index, index_within_archetype)
        }
        self.exact_component_ids_to_archetype
            .insert(component_ids.into(), archetype_index);
    }

    pub(crate) fn get_exact_archetype(&self, component_ids: &[ComponentId]) -> Option<usize> {
        self.exact_component_ids_to_archetype
            .get(component_ids)
            .cloned()
    }

    /// Iterate matching [Archetype]s
    /// The indices of [Archetype]s returned are guaranteed to be in increasing order.
    pub(crate) fn matching_archetype_iter<const FILTER_COUNT: usize>(
        &self,
        filters: &[Filter],
    ) -> MatchingArchetypeIterator<FILTER_COUNT> {
        let mut filter_info = [FilterInfo {
            filter_type: FilterType::With,
            component_id_to_archetypes: None,
        }; FILTER_COUNT];

        for (filter, filter_info) in filters.iter().zip(filter_info.iter_mut()) {
            filter_info.filter_type = filter.filter_type;
            if let Some(component_id_to_archetypes) =
                self.component_id_to_archetypes.get(&filter.component_id)
            {
                filter_info.component_id_to_archetypes = Some(component_id_to_archetypes);
            }
        }

        // Sort so the most restrictive filters are searched first.
        filter_info.sort_by_key(|f| match f.filter_type {
            FilterType::With => f.component_id_to_archetypes.map_or(0, |f| f.len()),
            FilterType::Without => {
                self.total_archetype_count - f.component_id_to_archetypes.map_or(0, |f| f.len())
            }
            FilterType::Optional => self.total_archetype_count,
        });

        MatchingArchetypeIterator {
            offset: 0,
            filter_info,
            total_archetypes_count: self.total_archetype_count,
        }
    }
}

#[derive(Copy, Clone)]
struct FilterInfo<'a> {
    filter_type: FilterType,
    component_id_to_archetypes: Option<&'a sparse_set::SparseSet<usize>>,
}

#[derive(Copy, Clone)]
pub struct Filter {
    pub component_id: ComponentId,
    pub filter_type: FilterType,
}

#[derive(Copy, Clone)]
pub enum FilterType {
    With,
    Without,
    Optional,
}

pub(crate) struct MatchingArchetypeIterator<'a, const FILTER_COUNT: usize> {
    offset: usize,
    filter_info: [FilterInfo<'a>; FILTER_COUNT],
    total_archetypes_count: usize,
}

impl<'a, const CHANNEL_COUNT: usize> Iterator for MatchingArchetypeIterator<'a, CHANNEL_COUNT> {
    /// Index to [Archetype] and the matching channel within the [Archetype]
    type Item = (usize, [Option<usize>; CHANNEL_COUNT]);

    fn next(&mut self) -> Option<Self::Item> {
        fn match_filters(
            archetype_index: usize,
            filter_info: &[FilterInfo],
            corresponding_channels: &mut [Option<usize>],
        ) -> Option<()> {
            for (filter, output_channel) in
                filter_info.iter().zip(corresponding_channels.iter_mut())
            {
                match filter.filter_type {
                    FilterType::With => {
                        let component_index_in_archetype =
                            filter.component_id_to_archetypes?.get(archetype_index)?;
                        *output_channel = Some(*component_index_in_archetype);
                    }
                    FilterType::Without => {
                        if let Some(component_id_to_archetype) = filter.component_id_to_archetypes {
                            if component_id_to_archetype.get(archetype_index).is_some() {
                                return None;
                            }
                        }
                    }
                    FilterType::Optional => {
                        if let Some(component_id_to_archetype) = filter.component_id_to_archetypes {
                            if let Some(component_index_in_archetype) =
                                component_id_to_archetype.get(archetype_index)
                            {
                                *output_channel = Some(*component_index_in_archetype)
                            }
                        }
                    }
                }
            }
            Some(())
        }

        let first_filter = self.filter_info[0];
        match self.filter_info[0].filter_type {
            FilterType::With => {
                if let Some(matching_archetypes) = first_filter.component_id_to_archetypes {
                    while let Some(&component_index_in_archetype) =
                        matching_archetypes.values().get(self.offset)
                    {
                        let archetype_index =
                            matching_archetypes.data_index_to_item_index()[self.offset];

                        let mut corresponding_channels = [None; CHANNEL_COUNT];
                        corresponding_channels[0] = Some(component_index_in_archetype);
                        self.offset += 1;

                        if match_filters(
                            archetype_index,
                            &self.filter_info[1..],
                            &mut corresponding_channels[1..],
                        )
                        .is_some()
                        {
                            return Some((archetype_index, corresponding_channels));
                        }
                    }
                }
            }
            FilterType::Without | FilterType::Optional => {
                let mut corresponding_channels = [None; CHANNEL_COUNT];
                for archetype_index in self.offset..self.total_archetypes_count {
                    if match_filters(
                        archetype_index,
                        &self.filter_info,
                        &mut corresponding_channels,
                    )
                    .is_some()
                    {
                        return Some((archetype_index, corresponding_channels));
                    }
                    self.offset += 1;
                }
            }
        }
        None
    }
}
