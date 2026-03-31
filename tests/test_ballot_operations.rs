//! Tests for Module 3: Ballot Operations domain types and registry.

use voteos::domain::ballots::*;

#[test]
fn test_create_ballot_template() {
    let registry = BallotRegistry::new();

    let id = registry.templates.insert_new(BallotTemplate {
        election_ref: "elec-001".into(),
        status: BallotStatus::Draft,
        items: Vec::new(),
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        finalized_at: None,
        finalized_by: None,
        decision_ref: "dec-001".into(),
        integrity_hash: None,
    });

    assert!(id.starts_with("btpl-"));
    let template = registry.templates.get(&id).expect("should exist");
    assert_eq!(template.status, BallotStatus::Draft);
    assert!(template.items.is_empty());
}

#[test]
fn test_add_ballot_items() {
    let registry = BallotRegistry::new();

    let id = registry.templates.insert_new(BallotTemplate {
        election_ref: "elec-001".into(),
        status: BallotStatus::Draft,
        items: Vec::new(),
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        finalized_at: None,
        finalized_by: None,
        decision_ref: "dec-001".into(),
        integrity_hash: None,
    });

    let mut template = registry.templates.get(&id).expect("exists");

    // Add a race
    template.items.push(BallotItem {
        item_ref: "item-001".into(),
        item_type: BallotItemType::Race,
        title: "Mayor".into(),
        description: "Election for City Mayor".into(),
        choices: vec![
            BallotChoice { choice_ref: "c-001".into(), label: "Alice Smith".into(), description: None },
            BallotChoice { choice_ref: "c-002".into(), label: "Bob Jones".into(), description: None },
        ],
        max_selections: 1,
    });

    // Add a measure
    template.items.push(BallotItem {
        item_ref: "item-002".into(),
        item_type: BallotItemType::Measure,
        title: "Proposition A".into(),
        description: "Increase park funding".into(),
        choices: vec![
            BallotChoice { choice_ref: "c-yes".into(), label: "Yes".into(), description: None },
            BallotChoice { choice_ref: "c-no".into(), label: "No".into(), description: None },
        ],
        max_selections: 1,
    });

    registry.templates.update(&id, template);

    let updated = registry.templates.get(&id).expect("exists");
    assert_eq!(updated.items.len(), 2);
    assert_eq!(updated.items[0].item_type, BallotItemType::Race);
    assert_eq!(updated.items[1].item_type, BallotItemType::Measure);
}

#[test]
fn test_remove_ballot_item() {
    let registry = BallotRegistry::new();

    let id = registry.templates.insert_new(BallotTemplate {
        election_ref: "elec-001".into(),
        status: BallotStatus::Draft,
        items: vec![
            BallotItem {
                item_ref: "item-001".into(),
                item_type: BallotItemType::Race,
                title: "Mayor".into(),
                description: "".into(),
                choices: vec![],
                max_selections: 1,
            },
            BallotItem {
                item_ref: "item-002".into(),
                item_type: BallotItemType::Measure,
                title: "Prop A".into(),
                description: "".into(),
                choices: vec![],
                max_selections: 1,
            },
        ],
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        finalized_at: None,
        finalized_by: None,
        decision_ref: "dec-001".into(),
        integrity_hash: None,
    });

    let mut template = registry.templates.get(&id).expect("exists");
    template.items.retain(|item| item.item_ref != "item-001");
    registry.templates.update(&id, template);

    let updated = registry.templates.get(&id).expect("exists");
    assert_eq!(updated.items.len(), 1);
    assert_eq!(updated.items[0].item_ref, "item-002");
}

#[test]
fn test_finalize_ballot() {
    let registry = BallotRegistry::new();

    let id = registry.templates.insert_new(BallotTemplate {
        election_ref: "elec-001".into(),
        status: BallotStatus::Draft,
        items: vec![BallotItem {
            item_ref: "item-001".into(),
            item_type: BallotItemType::Race,
            title: "Mayor".into(),
            description: "".into(),
            choices: vec![
                BallotChoice { choice_ref: "c-001".into(), label: "Alice".into(), description: None },
            ],
            max_selections: 1,
        }],
        created_by: "official-001".into(),
        created_at: "2026-03-30T10:00:00Z".into(),
        finalized_at: None,
        finalized_by: None,
        decision_ref: "dec-001".into(),
        integrity_hash: None,
    });

    let mut template = registry.templates.get(&id).expect("exists");
    let hash = BallotRegistry::compute_integrity_hash(&template);
    template.status = BallotStatus::Finalized;
    template.finalized_at = Some("2026-03-30T11:00:00Z".into());
    template.finalized_by = Some("official-001".into());
    template.integrity_hash = Some(hash.clone());
    registry.templates.update(&id, template);

    let finalized = registry.templates.get(&id).expect("exists");
    assert_eq!(finalized.status, BallotStatus::Finalized);
    assert!(finalized.integrity_hash.is_some());

    // Verify integrity
    let recomputed = BallotRegistry::compute_integrity_hash(&finalized);
    assert_eq!(recomputed, hash);
}

#[test]
fn test_integrity_hash_changes_with_content() {
    let template1 = BallotTemplate {
        election_ref: "elec-001".into(),
        status: BallotStatus::Draft,
        items: vec![BallotItem {
            item_ref: "item-001".into(),
            item_type: BallotItemType::Race,
            title: "Mayor".into(),
            description: "".into(),
            choices: vec![BallotChoice { choice_ref: "c-001".into(), label: "Alice".into(), description: None }],
            max_selections: 1,
        }],
        created_by: "o".into(), created_at: "t".into(),
        finalized_at: None, finalized_by: None, decision_ref: "d".into(), integrity_hash: None,
    };

    let template2 = BallotTemplate {
        election_ref: "elec-001".into(),
        status: BallotStatus::Draft,
        items: vec![BallotItem {
            item_ref: "item-001".into(),
            item_type: BallotItemType::Race,
            title: "Mayor".into(),
            description: "".into(),
            choices: vec![
                BallotChoice { choice_ref: "c-001".into(), label: "Alice".into(), description: None },
                BallotChoice { choice_ref: "c-002".into(), label: "Bob".into(), description: None },
            ],
            max_selections: 1,
        }],
        created_by: "o".into(), created_at: "t".into(),
        finalized_at: None, finalized_by: None, decision_ref: "d".into(), integrity_hash: None,
    };

    let hash1 = BallotRegistry::compute_integrity_hash(&template1);
    let hash2 = BallotRegistry::compute_integrity_hash(&template2);
    assert_ne!(hash1, hash2, "Different content must produce different hashes");
}

#[test]
fn test_ballot_issuance() {
    let registry = BallotRegistry::new();

    // Issue ballot
    let issuance_id = registry.issuances.insert_new(BallotIssuance {
        template_ref: "btpl-001".into(),
        voter_ref: "voter-001".into(),
        election_ref: "elec-001".into(),
        status: IssuanceStatus::Issued,
        issued_at: "2026-03-30T10:00:00Z".into(),
        issued_by: "official-001".into(),
        decision_ref: "dec-001".into(),
        spoiled_at: None,
        replacement_ref: None,
    });

    assert!(issuance_id.starts_with("biss-"));
    assert!(registry.has_active_issuance("voter-001", "elec-001"));
    assert!(!registry.has_active_issuance("voter-002", "elec-001"));
}

#[test]
fn test_prevent_duplicate_issuance() {
    let registry = BallotRegistry::new();

    registry.issuances.insert_new(BallotIssuance {
        template_ref: "btpl-001".into(),
        voter_ref: "voter-001".into(),
        election_ref: "elec-001".into(),
        status: IssuanceStatus::Issued,
        issued_at: "2026-03-30T10:00:00Z".into(),
        issued_by: "official-001".into(),
        decision_ref: "dec-001".into(),
        spoiled_at: None,
        replacement_ref: None,
    });

    // Second issuance should be caught by has_active_issuance check
    assert!(registry.has_active_issuance("voter-001", "elec-001"));
}

#[test]
fn test_revoke_ballot() {
    let registry = BallotRegistry::new();

    let id = registry.issuances.insert_new(BallotIssuance {
        template_ref: "btpl-001".into(),
        voter_ref: "voter-001".into(),
        election_ref: "elec-001".into(),
        status: IssuanceStatus::Issued,
        issued_at: "2026-03-30T10:00:00Z".into(),
        issued_by: "official-001".into(),
        decision_ref: "dec-001".into(),
        spoiled_at: None,
        replacement_ref: None,
    });

    assert!(registry.has_active_issuance("voter-001", "elec-001"));

    // Spoil
    let mut issuance = registry.issuances.get(&id).expect("exists");
    issuance.status = IssuanceStatus::Spoiled;
    issuance.spoiled_at = Some("2026-03-30T11:00:00Z".into());
    registry.issuances.update(&id, issuance);

    assert!(!registry.has_active_issuance("voter-001", "elec-001"));
}

#[test]
fn test_issuance_count() {
    let registry = BallotRegistry::new();

    for i in 1..=5 {
        registry.issuances.insert_new(BallotIssuance {
            template_ref: "btpl-001".into(),
            voter_ref: format!("voter-{:03}", i),
            election_ref: "elec-001".into(),
            status: IssuanceStatus::Issued,
            issued_at: "2026-03-30T10:00:00Z".into(),
            issued_by: "official-001".into(),
            decision_ref: format!("dec-{:03}", i),
            spoiled_at: None,
            replacement_ref: None,
        });
    }

    assert_eq!(registry.issuance_count("elec-001"), 5);
    assert_eq!(registry.issuance_count("elec-002"), 0);
}

#[test]
fn test_template_for_election() {
    let registry = BallotRegistry::new();

    registry.templates.insert_new(BallotTemplate {
        election_ref: "elec-001".into(),
        status: BallotStatus::Finalized,
        items: vec![],
        created_by: "o".into(), created_at: "t".into(),
        finalized_at: Some("t".into()), finalized_by: Some("o".into()),
        decision_ref: "d".into(), integrity_hash: Some("h".into()),
    });

    let found = registry.finalized_template("elec-001");
    assert!(found.is_some());
    assert!(registry.finalized_template("elec-999").is_none());
}

#[test]
fn test_persistence_roundtrip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path();

    {
        let registry = BallotRegistry::with_data_dir(path);
        registry.templates.insert_new(BallotTemplate {
            election_ref: "elec-001".into(),
            status: BallotStatus::Draft,
            items: vec![BallotItem {
                item_ref: "item-001".into(),
                item_type: BallotItemType::Race,
                title: "Mayor".into(),
                description: "".into(),
                choices: vec![],
                max_selections: 1,
            }],
            created_by: "o".into(), created_at: "t".into(),
            finalized_at: None, finalized_by: None, decision_ref: "d".into(), integrity_hash: None,
        });
    }

    {
        let registry = BallotRegistry::with_data_dir(path);
        assert_eq!(registry.templates.count(), 1);
        let all = registry.templates.find_all(|_| true);
        assert_eq!(all[0].1.items.len(), 1);
    }
}

#[test]
fn test_audit_trail() {
    let registry = BallotRegistry::new();

    registry.audit_log.insert_new(BallotAuditEntry {
        action: "create_ballot_template".into(),
        actor_ref: "official-001".into(),
        target_ref: Some("btpl-001".into()),
        election_ref: "elec-001".into(),
        timestamp: "2026-03-30T10:00:00Z".into(),
        decision_ref: "dec-001".into(),
        details: "Created template".into(),
    });

    let all = registry.audit_log.find_all(|_| true);
    assert_eq!(all.len(), 1);
}

// ---------------------------------------------------------------------------
// Workflow tests (require AxiaSystem replica)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires local ICP replica"]
async fn test_ballot_lifecycle_workflow_strict() {
    // STRICT_HAPPY_PATH_BLOCKED (environment)
}
