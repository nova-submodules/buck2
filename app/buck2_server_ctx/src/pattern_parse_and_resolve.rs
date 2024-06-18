/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

use buck2_core::fs::project_rel_path::ProjectRelativePath;
use buck2_core::pattern::pattern_type::PatternType;
use buck2_core::pattern::pattern_type::ProvidersPatternExtra;
use buck2_core::provider::label::ProvidersLabel;
use buck2_core::target::label::label::TargetLabel;
use buck2_node::nodes::frontend::TargetGraphCalculation;
use dice::DiceComputations;
use dupe::Dupe;
use gazebo::prelude::VecExt;

use crate::pattern;

pub async fn parse_and_resolve_patterns_to_targets_from_cli_args<T: PatternType>(
    ctx: &mut DiceComputations<'_>,
    target_patterns: &[String],
    cwd: &ProjectRelativePath,
) -> anyhow::Result<Vec<(TargetLabel, T)>> {
    let resolved_pattern =
        pattern::parse_and_resolve_patterns_from_cli_args::<T>(ctx, target_patterns, cwd).await?;
    let mut result_targets = Vec::new();
    for (package, spec) in resolved_pattern.specs {
        match spec {
            buck2_core::pattern::PackageSpec::Targets(targets) => {
                result_targets.extend(targets.into_map(|(name, extra)| {
                    (TargetLabel::new(package.dupe(), name.as_ref()), extra)
                }))
            }
            buck2_core::pattern::PackageSpec::All => {
                // Note this code is not parallel. Careful if used in performance sensitive code.
                let interpreter_results = ctx.get_interpreter_results(package.dupe()).await?;
                result_targets.extend(
                    interpreter_results
                        .targets()
                        .keys()
                        .map(|target| (TargetLabel::new(package.dupe(), target), T::default())),
                );
            }
        }
    }
    Ok(result_targets)
}

pub async fn parse_and_resolve_provider_labels_from_cli_args(
    ctx: &mut DiceComputations<'_>,
    target_patterns: &[String],
    cwd: &ProjectRelativePath,
) -> anyhow::Result<Vec<ProvidersLabel>> {
    let targets = parse_and_resolve_patterns_to_targets_from_cli_args::<ProvidersPatternExtra>(
        ctx,
        target_patterns,
        cwd,
    )
    .await?;
    Ok(targets
        .into_map(|(label, providers)| providers.into_providers_label(label.pkg(), label.name())))
}
