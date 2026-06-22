use indexmap::IndexMap;

use super::{StaticDeclaration, SxDyn, class_name_from_id};

#[derive(Clone, PartialEq, Eq)]
pub struct OptimizedRule {
    pub selector: String,
    pub sx: SxDyn,
}

fn declaration_key(declaration: &StaticDeclaration) -> String {
    declaration.to_string()
}

fn static_only_sx(declarations: Vec<StaticDeclaration>) -> SxDyn {
    SxDyn {
        declarations,
        dynamic_declarations: None,
        nested_rules: Vec::new(),
    }
}

pub fn optimize_styles(registry: &IndexMap<String, SxDyn>) -> Vec<OptimizedRule> {
    let mut optimized_rules = Vec::new();
    let mut exact_deduped: IndexMap<String, (String, SxDyn)> = IndexMap::new();

    for (id, sx) in registry {
        let css_key = sx.to_string();
        exact_deduped
            .entry(css_key)
            .or_insert_with(|| (id.clone(), sx.clone()));
    }

    let deduped_rules: Vec<(String, SxDyn)> = exact_deduped.into_values().collect();
    let mut static_frequency: IndexMap<String, (StaticDeclaration, Vec<usize>)> = IndexMap::new();

    for (rule_index, (_, sx)) in deduped_rules.iter().enumerate() {
        for declaration in &sx.declarations {
            let key = declaration_key(declaration);
            let entry = static_frequency
                .entry(key)
                .or_insert_with(|| (declaration.clone(), Vec::new()));

            if !entry.1.contains(&rule_index) {
                entry.1.push(rule_index);
            }
        }
    }

    let shared_entries: Vec<(StaticDeclaration, Vec<usize>)> = static_frequency
        .into_values()
        .filter(|(_, rule_indices)| rule_indices.len() > 1)
        .collect();

    let shared_declarations: Vec<StaticDeclaration> = shared_entries
        .iter()
        .map(|(declaration, _)| declaration.clone())
        .collect();

    if !shared_entries.is_empty() {
        let grouped_selector = deduped_rules
            .iter()
            .enumerate()
            .filter(|(rule_index, _)| {
                shared_entries
                    .iter()
                    .any(|(_, rule_indices)| rule_indices.contains(rule_index))
            })
            .map(|(_, (id, _))| format!(".{}", class_name_from_id(id)))
            .collect::<Vec<_>>()
            .join(", ");

        optimized_rules.push(OptimizedRule {
            selector: grouped_selector,
            sx: static_only_sx(shared_declarations.clone()),
        });
    }

    for (id, sx) in deduped_rules {
        let declarations = sx
            .declarations
            .into_iter()
            .filter(|declaration| !shared_declarations.contains(declaration))
            .collect::<Vec<_>>();

        let has_static_declarations = !declarations.is_empty();
        let has_dynamic_declarations = sx
            .dynamic_declarations
            .as_ref()
            .is_some_and(|dynamic| !dynamic.is_empty());

        if has_static_declarations || has_dynamic_declarations {
            optimized_rules.push(OptimizedRule {
                selector: format!(".{}", class_name_from_id(&id)),
                sx: SxDyn {
                    declarations,
                    dynamic_declarations: sx.dynamic_declarations,
                    nested_rules: sx.nested_rules,
                },
            });
        }
    }

    optimized_rules
}
