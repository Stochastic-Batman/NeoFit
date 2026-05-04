use {
    anchor_lang::{
        prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize,
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    neofit::state::UserProfile,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const DAY: i64 = 86_400;

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/neofit.so");
    svm.add_program(neofit::id(), bytes).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

fn derive_user_profile(authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"user_profile", authority.as_ref()],
        &neofit::id(),
    )
    .0
}

fn initialize_user(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
    let pda = derive_user_profile(&payer.pubkey());
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::InitializeUser {}.data(),
        neofit::accounts::InitializeUser {
            user_profile: pda,
            authority: payer.pubkey(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).unwrap();
    pda
}

#[allow(dead_code)]
fn log_reps(svm: &mut LiteSVM, payer: &Keypair, exercise_id: u8, count: u32) -> bool {
    let pda = derive_user_profile(&payer.pubkey());
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::LogReps { exercise_id, count }.data(),
        neofit::accounts::LogReps {
            user_profile: pda,
            enrollment: None,
            challenge: None,
            authority: payer.pubkey(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).is_ok()
}

fn log_reps_ok(svm: &mut LiteSVM, payer: &Keypair, exercise_id: u8, count: u32) {
    let pda = derive_user_profile(&payer.pubkey());
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::LogReps { exercise_id, count }.data(),
        neofit::accounts::LogReps {
            user_profile: pda,
            enrollment: None,
            challenge: None,
            authority: payer.pubkey(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    svm.send_transaction(tx).expect("log_reps should succeed");
}

fn get_profile(svm: &LiteSVM, pda: &Pubkey) -> UserProfile {
    let raw = svm.get_account(pda).expect("UserProfile must exist");
    UserProfile::try_deserialize(&mut raw.data.as_ref()).unwrap()
}

fn set_clock(svm: &mut LiteSVM, unix_timestamp: i64) {
    let clock = anchor_lang::solana_program::clock::Clock {
        unix_timestamp,
        ..anchor_lang::solana_program::clock::Clock::default()
    };
    svm.set_sysvar(&clock);
}

#[test]
fn test_log_reps_increments_total_reps() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    log_reps_ok(&mut svm, &payer, 0, 10);
    log_reps_ok(&mut svm, &payer, 0, 5);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.total_reps, 15);
}

#[test]
fn test_log_reps_creates_sparse_rep_count_entry() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    log_reps_ok(&mut svm, &payer, 1, 20);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.rep_counts.len(), 1);
    assert_eq!(profile.rep_counts[0].exercise_id, 1);
    assert_eq!(profile.rep_counts[0].count, 20);
}

#[test]
fn test_log_reps_accumulates_per_exercise() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    log_reps_ok(&mut svm, &payer, 0, 10);
    log_reps_ok(&mut svm, &payer, 1, 20);
    log_reps_ok(&mut svm, &payer, 0, 5);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.total_reps, 35);
    assert_eq!(profile.rep_counts.len(), 2);

    let squat = profile.rep_counts.iter().find(|e| e.exercise_id == 0).unwrap();
    let pushup = profile.rep_counts.iter().find(|e| e.exercise_id == 1).unwrap();
    assert_eq!(squat.count, 15);
    assert_eq!(pushup.count, 20);
}

#[test]
fn test_log_reps_invalid_exercise_id_fails() {
    let (mut svm, payer) = setup();
    initialize_user(&mut svm, &payer);

    let pda = derive_user_profile(&payer.pubkey());
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::LogReps { exercise_id: 5, count: 10 }.data(),
        neofit::accounts::LogReps {
            user_profile: pda,
            enrollment: None,
            challenge: None,
            authority: payer.pubkey(),
        }
        .to_account_metas(None),
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    assert!(
        svm.send_transaction(tx).is_err(),
        "exercise_id >= MAX_EXERCISE_ID must be rejected"
    );
}

#[test]
fn test_streak_first_workout_sets_one() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    set_clock(&mut svm, DAY * 100);
    log_reps_ok(&mut svm, &payer, 0, 10);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.streak_days, 1);
    assert_eq!(profile.last_workout_ts, DAY * 100);
}

#[test]
fn test_streak_same_day_unchanged() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    set_clock(&mut svm, DAY * 100);
    log_reps_ok(&mut svm, &payer, 0, 10);

    set_clock(&mut svm, DAY * 100 + 3600);
    log_reps_ok(&mut svm, &payer, 0, 10);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.streak_days, 1, "same-day second workout must not increment streak");
}

#[test]
fn test_streak_next_day_increments() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    set_clock(&mut svm, DAY * 100);
    log_reps_ok(&mut svm, &payer, 0, 10);

    set_clock(&mut svm, DAY * 101);
    log_reps_ok(&mut svm, &payer, 0, 10);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.streak_days, 2);
}

#[test]
fn test_streak_gap_resets_to_one() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    set_clock(&mut svm, DAY * 100);
    log_reps_ok(&mut svm, &payer, 0, 10);

    set_clock(&mut svm, DAY * 101);
    log_reps_ok(&mut svm, &payer, 0, 10);

    set_clock(&mut svm, DAY * 102);
    log_reps_ok(&mut svm, &payer, 0, 10);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.streak_days, 3);

    set_clock(&mut svm, DAY * 105);
    log_reps_ok(&mut svm, &payer, 0, 10);

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.streak_days, 1, "missing a day must reset streak to 1");
}

#[test]
fn test_streak_multi_day_build() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    for day in 0..7 {
        set_clock(&mut svm, DAY * (200 + day));
        log_reps_ok(&mut svm, &payer, 0, 10);
    }

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.streak_days, 7);
}

#[test]
fn test_log_reps_all_valid_exercise_ids() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    for id in 0..5u8 {
        log_reps_ok(&mut svm, &payer, id, 1);
    }

    let profile = get_profile(&svm, &pda);
    assert_eq!(profile.total_reps, 5);
    assert_eq!(profile.rep_counts.len(), 5);
}
