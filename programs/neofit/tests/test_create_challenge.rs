use {
    anchor_lang::{
        prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize,
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    neofit::state::{Challenge, ExerciseRequirement},
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

const FUTURE_DEADLINE: i64 = 9_999_999_999;

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/neofit.so");
    svm.add_program(neofit::id(), bytes).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

fn derive_challenge(authority: &Pubkey, nonce: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"challenge", authority.as_ref(), &nonce.to_le_bytes()],
        &neofit::id(),
    )
    .0
}

fn create_challenge_ix(
    authority: &Keypair,
    nonce: u64,
    title: &str,
    requirements: Vec<ExerciseRequirement>,
    entry_fee: u64,
    deadline: i64,
) -> Instruction {
    let pda = derive_challenge(&authority.pubkey(), nonce);
    Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::CreateChallenge {
            title: title.to_string(),
            requirements,
            entry_fee,
            deadline,
            nonce,
        }
        .data(),
        neofit::accounts::CreateChallenge {
            challenge: pda,
            authority: authority.pubkey(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn get_challenge(svm: &LiteSVM, pda: &Pubkey) -> Challenge {
    let raw = svm.get_account(pda).expect("Challenge account must exist");
    Challenge::try_deserialize(&mut raw.data.as_ref()).unwrap()
}

fn single_req(exercise_id: u8, rep_target: u16) -> Vec<ExerciseRequirement> {
    vec![ExerciseRequirement { exercise_id, rep_target }]
}

#[test]
fn test_create_challenge_basic() {
    let (mut svm, payer) = setup();
    let pda = derive_challenge(&payer.pubkey(), 0);

    let ix = create_challenge_ix(&payer, 0, "Iron Week", single_req(0, 100), 0, FUTURE_DEADLINE);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let challenge = get_challenge(&svm, &pda);
    assert_eq!(challenge.authority, payer.pubkey());
    assert_eq!(challenge.title, "Iron Week");
    assert_eq!(challenge.requirements.len(), 1);
    assert_eq!(challenge.requirements[0].exercise_id, 0);
    assert_eq!(challenge.requirements[0].rep_target, 100);
    assert_eq!(challenge.entry_fee_lamports, 0);
    assert_eq!(challenge.pool_lamports, 0);
    assert_eq!(challenge.completers, 0);
    assert_eq!(challenge.deadline_ts, FUTURE_DEADLINE);
    assert!(challenge.is_active);
    assert_eq!(challenge.nonce, 0);
}

#[test]
fn test_create_challenge_with_entry_fee() {
    let (mut svm, payer) = setup();
    let pda = derive_challenge(&payer.pubkey(), 0);

    let ix = create_challenge_ix(
        &payer, 0, "Paid Grind", single_req(1, 50),
        100_000_000, FUTURE_DEADLINE,
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let challenge = get_challenge(&svm, &pda);
    assert_eq!(challenge.entry_fee_lamports, 100_000_000);
}

#[test]
fn test_create_challenge_multi_requirement() {
    let (mut svm, payer) = setup();
    let pda = derive_challenge(&payer.pubkey(), 0);

    let requirements = vec![
        ExerciseRequirement { exercise_id: 0, rep_target: 100 },
        ExerciseRequirement { exercise_id: 1, rep_target: 50 },
        ExerciseRequirement { exercise_id: 4, rep_target: 200 },
    ];
    let ix = create_challenge_ix(&payer, 0, "Full Body", requirements.clone(), 0, FUTURE_DEADLINE);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let challenge = get_challenge(&svm, &pda);
    assert_eq!(challenge.requirements.len(), 3);
    assert_eq!(challenge.requirements[1].exercise_id, 1);
    assert_eq!(challenge.requirements[1].rep_target, 50);
}

#[test]
fn test_create_challenge_nonce_uniqueness() {
    let (mut svm, payer) = setup();

    for nonce in 0..3u64 {
        let ix = create_challenge_ix(
            &payer, nonce, "Same Title", single_req(0, 10), 0, FUTURE_DEADLINE,
        );
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
        let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
        assert!(svm.send_transaction(tx).is_ok(), "nonce {} must produce a fresh PDA", nonce);
    }

    for nonce in 0..3u64 {
        let pda = derive_challenge(&payer.pubkey(), nonce);
        let challenge = get_challenge(&svm, &pda);
        assert_eq!(challenge.nonce, nonce);
    }
}

#[test]
fn test_create_challenge_any_wallet_can_create() {
    let (mut svm, _admin) = setup();
    let random_user = Keypair::new();
    svm.airdrop(&random_user.pubkey(), 10_000_000_000).unwrap();

    let ix = create_challenge_ix(&random_user, 0, "Open Challenge", single_req(0, 10), 0, FUTURE_DEADLINE);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&random_user.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&random_user]).unwrap();
    assert!(svm.send_transaction(tx).is_ok(), "any wallet must be able to create a challenge");
}

#[test]
fn test_create_challenge_deadline_in_past_fails() {
    let (mut svm, payer) = setup();

    let ix = create_challenge_ix(&payer, 0, "Expired", single_req(0, 10), 0, -1);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "deadline in the past must be rejected"
    );
}

#[test]
fn test_create_challenge_empty_requirements_fails() {
    let (mut svm, payer) = setup();

    let ix = create_challenge_ix(&payer, 0, "Empty", vec![], 0, FUTURE_DEADLINE);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "empty requirements must be rejected"
    );
}

#[test]
fn test_create_challenge_too_many_requirements_fails() {
    let (mut svm, payer) = setup();

    let requirements = (0..16)
        .map(|i| ExerciseRequirement { exercise_id: (i % 5) as u8, rep_target: 10 })
        .collect();
    let ix = create_challenge_ix(&payer, 0, "Overloaded", requirements, 0, FUTURE_DEADLINE);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "more than MAX_REQUIREMENTS entries must be rejected"
    );
}

#[test]
fn test_create_challenge_invalid_exercise_id_fails() {
    let (mut svm, payer) = setup();

    let ix = create_challenge_ix(
        &payer, 0, "Bad Exercise",
        vec![ExerciseRequirement { exercise_id: 5, rep_target: 10 }],
        0, FUTURE_DEADLINE,
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
fn test_create_challenge_zero_rep_target_fails() {
    let (mut svm, payer) = setup();

    let ix = create_challenge_ix(
        &payer, 0, "Zero Reps",
        vec![ExerciseRequirement { exercise_id: 0, rep_target: 0 }],
        0, FUTURE_DEADLINE,
    );
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "rep_target of zero must be rejected"
    );
}

#[test]
fn test_create_challenge_max_requirements_accepted() {
    let (mut svm, payer) = setup();

    let requirements = (0..15)
        .map(|i| ExerciseRequirement { exercise_id: (i % 5) as u8, rep_target: 10 })
        .collect();
    let ix = create_challenge_ix(&payer, 0, "Max Reqs", requirements, 0, FUTURE_DEADLINE);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(svm.send_transaction(tx).is_ok(), "exactly MAX_REQUIREMENTS must be accepted");

    let pda = derive_challenge(&payer.pubkey(), 0);
    let challenge = get_challenge(&svm, &pda);
    assert_eq!(challenge.requirements.len(), 15);
}
