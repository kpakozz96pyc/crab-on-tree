use crabontree_app::{AppMessage, BranchTreeState};
use eframe::egui;

pub enum BranchesAction {
    None,
    SelectBranch { name: String, is_remote: bool },
    CheckoutBranch { name: String, is_remote: bool },
}

fn render_local_section(
    ui: &mut egui::Ui,
    branches: &[crabontree_app::BranchInfo],
    current_branch: &str,
    selected_branch: Option<&String>,
    action: &mut BranchesAction,
) {
    if branches.is_empty() {
        return;
    }

    egui::CollapsingHeader::new(format!("LOCAL ({})", branches.len()))
        .id_source("branches_local")
        .default_open(true)
        .show(ui, |ui| {
            for (idx, branch) in branches.iter().enumerate() {
                ui.push_id(format!("local_{}", idx), |ui| {
                    let is_current = branch.is_current || branch.name == current_branch;
                    let is_selected = selected_branch == Some(&branch.name);

                    let response = ui.horizontal(|ui| {
                        // Indicator for current branch
                        if is_current {
                            ui.label("*");
                        } else {
                            ui.label(" ");
                        }

                        // Branch icon
                        ui.label(">");

                        // Branch name
                        let text = if is_current {
                            egui::RichText::new(&branch.name).strong()
                        } else {
                            egui::RichText::new(&branch.name)
                        };

                        ui.selectable_label(is_selected, text)
                    });

                    // Single-click: select branch
                    if response.inner.clicked() {
                        *action = BranchesAction::SelectBranch {
                            name: branch.name.clone(),
                            is_remote: false,
                        };
                    }

                    // Double-click: checkout branch
                    if response.inner.double_clicked() && !is_current {
                        *action = BranchesAction::CheckoutBranch {
                            name: branch.name.clone(),
                            is_remote: false,
                        };
                    }
                });
            }
        });
}

fn render_remote_section(
    ui: &mut egui::Ui,
    remotes: &std::collections::HashMap<String, Vec<crabontree_app::BranchInfo>>,
    selected_branch: Option<&String>,
    action: &mut BranchesAction,
) {
    if remotes.is_empty() {
        return;
    }

    // Count total remote branches
    let total_count: usize = remotes.values().map(|v| v.len()).sum();

    egui::CollapsingHeader::new(format!("REMOTE ({})", total_count))
        .id_source("branches_remote")
        .default_open(true)
        .show(ui, |ui| {
            for (remote_name, branches) in remotes.iter() {
                // Show remote name as a sub-header
                egui::CollapsingHeader::new(format!("☁ {} ({})", remote_name, branches.len()))
                    .id_source(format!("remote_{}", remote_name))
                    .default_open(true)
                    .show(ui, |ui| {
                        for (idx, branch) in branches.iter().enumerate() {
                            ui.push_id(format!("remote_{}_{}", remote_name, idx), |ui| {
                                // Create full remote branch name (remote/branch)
                                let full_name = format!("{}/{}", remote_name, branch.name);
                                let is_selected = selected_branch.as_ref().map(|s| s.as_str()) == Some(&full_name);

                                let response = ui.horizontal(|ui| {
                                    ui.label(" ");  // Space for alignment with local branches
                                    ui.label("🌿");
                                    ui.selectable_label(is_selected, &branch.name)
                                });

                                // Single-click: select branch
                                if response.inner.clicked() {
                                    *action = BranchesAction::SelectBranch {
                                        name: full_name.clone(),
                                        is_remote: true,
                                    };
                                }

                                // Double-click: checkout remote branch
                                if response.inner.double_clicked() {
                                    *action = BranchesAction::CheckoutBranch {
                                        name: full_name,
                                        is_remote: true,
                                    };
                                }
                            });
                        }
                    });
            }
        });
}

pub fn render(ui: &mut egui::Ui, branch_tree: &BranchTreeState) -> BranchesAction {
    let mut action = BranchesAction::None;

    // Handle Enter key on selected branch
    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        if let Some(selected) = &branch_tree.selected_branch {
            // Determine if it's a remote branch (contains '/')
            let is_remote = selected.contains('/');
            // Don't checkout if it's the current branch
            let is_current = !is_remote && selected == &branch_tree.current_branch;

            if !is_current {
                action = BranchesAction::CheckoutBranch {
                    name: selected.clone(),
                    is_remote,
                };
            }
        }
    }

    // Filter icon and count
    ui.horizontal(|ui| {
        ui.label("👁");
        ui.label(format!(
            "Viewing {}",
            branch_tree.local_branches.len() +
            branch_tree.remote_branches.values().map(|v| v.len()).sum::<usize>()
        ));
    });

    ui.add_space(5.0);

    // Local branches section
    if !branch_tree.local_branches.is_empty() {
        render_local_section(
            ui,
            &branch_tree.local_branches,
            &branch_tree.current_branch,
            branch_tree.selected_branch.as_ref(),
            &mut action,
        );
        ui.add_space(5.0);
    }

    // Remote branches section
    if !branch_tree.remote_branches.is_empty() {
        render_remote_section(
            ui,
            &branch_tree.remote_branches,
            branch_tree.selected_branch.as_ref(),
            &mut action,
        );
    }

    action
}

pub fn action_to_message(action: BranchesAction) -> Option<AppMessage> {
    match action {
        BranchesAction::None => None,
        BranchesAction::SelectBranch { name, is_remote } => {
            Some(AppMessage::BranchSelected { name, is_remote })
        }
        BranchesAction::CheckoutBranch { name, is_remote } => {
            Some(AppMessage::BranchCheckoutRequested { name, is_remote })
        }
    }
}
