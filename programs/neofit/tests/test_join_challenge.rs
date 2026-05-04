use {
    anchor_lang::{
        prelude::Pubkey, solana_program::instruction::Instruction, AccountDeserialize,
        InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    neofit::state::{Challenge, Enrollment, ExerciseRequirement},
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

fn new_funded_user(svm: &mut LiteSVM) -> Keypair {
    let kp = Keypair::new();
    svm.airdrop(&kp.pubkey(), 10_000_000_000).unwrap();
    kp
}

fn send(svm: &mut LiteSVM, ix: Instruction, signer: &Keypair) -> bool {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&signer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
    svm.send_transaction(tx).is_ok()
}

fn derive_user_profile(authority: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"user_profile", authority.as_ref()], &neofit::id()).0
}

fn derive_challenge(authority: &Pubkey, nonce: u64) -> Pubkey {
    Pubkey::find_program_address(
        &[b"challenge", authority.as_ref(), &nonce.to_le_bytes()],
        &neofit::id(),
    )
    .0
}

fn derive_enrollment(challenge: &Pubkey, user: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"enrollment", challenge.as_ref(), user.as_ref()],
        &neofit::id(),
    )
    .0
}

fn initialize_user(svm: &mut LiteSVM, user: &Keypair) -> Pubkey {
    let pda = derive_user_profile(&user.pubkey());
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::InitializeUser {}.data(),
        neofit::accounts::InitializeUser {
            user_profile: pda,
            authority: user.pubkey(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(svm, ix, user));
    pda
}

fn create_challenge(
    svm: &mut LiteSVM,
    creator: &Keypair,
    nonce: u64,
    requirements: Vec<ExerciseRequirement>,
    entry_fee: u64,
    deadline: i64,
) -> Pubkey {
    let pda = derive_challenge(&creator.pubkey(), nonce);
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::CreateChallenge {
            title: "Test Challenge".to_string(),
            requirements,
            entry_fee,
            deadline,
            nonce,
        }
        .data(),
        neofit::accounts::CreateChallenge {
            challenge: pda,
            authority: creator.pubkey(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    );
    assert!(send(svm, ix, creator));
    pda
}

fn join_challenge_ix(
    user_profile: Pubkey,
    challenge: Pubkey,
    enrollment: Pubkey,
    authority: Pubkey,
) -> Instruction {
    Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::JoinChallenge {}.data(),
        neofit::accounts::JoinChallenge {
            enrollment,
            challenge,
            user_profile,
            authority,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn get_enrollment(svm: &LiteSVM, pda: &Pubkey) -> Enrollment {
    let raw = svm.get_account(pda).expect("Enrollment account must exist");
    Enrollment::try_deserialize(&mut raw.data.as_ref()).unwrap()
}

fn get_challenge(svm: &LiteSVM, pda: &Pubkey) -> Challenge {
    let raw = svm.get_account(pda).expect("Challenge account must exist");
    Challenge::try_deserialize(&mut raw.data.as_ref()).unwrap()
}

fn single_req(exercise_id: u8, rep_target: u16) -> Vec<ExerciseRequirement> {
    vec![ExerciseRequirement { exercise_id, rep_target }]
}

#[test]
fn test_join_creates_enrollment() {
    let (mut svm, user) = setup();
    let profile = initialize_user(&mut svm, &user);
    let challenge_pda = create_challenge(&mut svm, &user, 0, single_req(0, 50), 0, FUTURE_DEADLINE);
    let enrollment_pda = derive_enrollment(&challenge_pda, &user.pubkey());

    let ix = join_challenge_ix(profile, challenge_pda, enrollment_pda, user.pubkey());
    assert!(send(&mut svm, ix, &user));

    let enrollment = get_enrollment(&svm, &enrollment_pda);
    assert_eq!(enrollment.user, user.pubkey());
    assert_eq!(enrollment.challenge, challenge_pda);
    assert!(!enrollment.completed);
    assert!(!enrollment.reward_claimed);
}

#[test]
fn test_join_reps_logged_prepopulated_matches_requirements() {
    let (mut svm, user) = setup();
    let profile = initialize_user(&mut svm, &user);
    let requirements = vec![
        ExerciseRequirement { exercise_id: 1, rep_target: 30 },
        ExerciseRequirement { exercise_id: 3, rep_target: 20 },
    ];
    let challenge_pda = create_challenge(&mut svm, &user, 0, requirements, 0, FUTURE_DEADLINE);
    let enrollment_pda = derive_enrollment(&challenge_pda, &user.pubkey());

    let ix = join_challenge_ix(profile, challenge_pda, enrollment_pda, user.pubkey());
    assert!(send(&mut svm, ix, &user));

    let enrollment = get_enrollment(&svm, &enrollment_pda);
    assert_eq!(enrollment.reps_logged.len(), 2);
    assert_eq!(enrollment.reps_logged[0].exercise_id, 1);
    assert_eq!(enrollment.reps_logged[0].count, 0);
    assert_eq!(enrollment.reps_logged[1].exercise_id, 3);
    assert_eq!(enrollment.reps_logged[1].count, 0);
}

#[test]
fn test_join_free_challenge_no_lamport_transfer() {
    let (mut svm, user) = setup();
    let profile = initialize_user(&mut svm, &user);
    let challenge_pda = create_challenge(&mut svm, &user, 0, single_req(0, 10), 0, FUTURE_DEADLINE);
    let enrollment_pda = derive_enrollment(&challenge_pda, &user.pubkey());

    let challenge_lamports_before = svm.get_account(&challenge_pda).unwrap().lamports;

    let ix = join_challenge_ix(profile, challenge_pda, enrollment_pda, user.pubkey());
    assert!(send(&mut svm, ix, &user));

    let challenge = get_challenge(&svm, &challenge_pda);
    assert_eq!(challenge.pool_lamports, 0);

    let challenge_lamports_after = svm.get_account(&challenge_pda).unwrap().lamports;
    assert_eq!(challenge_lamports_before, challenge_lamports_after);
}

#[test]
fn test_join_paid_challenge_transfers_fee() {
    let (mut svm, creator) = setup();
    let joiner = new_funded_user(&mut svm);

    let entry_fee = 500_000_000u64;
    initialize_user(&mut svm, &creator);
    initialize_user(&mut svm, &joiner);

    let challenge_pda = create_challenge(&mut svm, &creator, 0, single_req(0, 10), entry_fee, FUTURE_DEADLINE);
    let joiner_profile = derive_user_profile(&joiner.pubkey());
    let enrollment_pda = derive_enrollment(&challenge_pda, &joiner.pubkey());

    let joiner_lamports_before = svm.get_account(&joiner.pubkey()).unwrap().lamports;
    let challenge_lamports_before = svm.get_account(&challenge_pda).unwrap().lamports;

    let ix = join_challenge_ix(joiner_profile, challenge_pda, enrollment_pda, joiner.pubkey());
    assert!(send(&mut svm, ix, &joiner));

    let joiner_lamports_after = svm.get_account(&joiner.pubkey()).unwrap().lamports;
    let challenge_lamports_after = svm.get_account(&challenge_pda).unwrap().lamports;

    assert!(joiner_lamports_after <= joiner_lamports_before - entry_fee);
    assert_eq!(challenge_lamports_after, challenge_lamports_before + entry_fee);

    let challenge = get_challenge(&svm, &challenge_pda);
    assert_eq!(challenge.pool_lamports, entry_fee);
}

#[test]
fn test_join_multiple_users_accumulates_pool() {
    let (mut svm, creator) = setup();
    let entry_fee = 100_000_000u64;
    initialize_user(&mut svm, &creator);
    let challenge_pda = create_challenge(&mut svm, &creator, 0, single_req(0, 10), entry_fee, FUTURE_DEADLINE);

    for _ in 0..3 {
        let joiner = new_funded_user(&mut svm);
        let profile = initialize_user(&mut svm, &joiner);
        let enrollment = derive_enrollment(&challenge_pda, &joiner.pubkey());
        let ix = join_challenge_ix(profile, challenge_pda, enrollment, joiner.pubkey());
        assert!(send(&mut svm, ix, &joiner));
    }

    let challenge = get_challenge(&svm, &challenge_pda);
    assert_eq!(challenge.pool_lamports, entry_fee * 3);
}

#[test]
fn test_join_expired_challenge_fails() {
    let (mut svm, user) = setup();
    let profile = initialize_user(&mut svm, &user);

    let soon: i64 = 1_000;
    let challenge_pda = create_challenge(&mut svm, &user, 0, single_req(0, 10), 0, soon);
    let enrollment_pda = derive_enrollment(&challenge_pda, &user.pubkey());

    let clock = anchor_lang::solana_program::clock::Clock {
        unix_timestamp: soon + 1,
        ..anchor_lang::solana_program::clock::Clock::default()
    };
    svm.set_sysvar(&clock);

    let ix = join_challenge_ix(profile, challenge_pda, enrollment_pda, user.pubkey());
    assert!(
        !send(&mut svm, ix, &user),
        "joining past the deadline must fail"
    );
}

#[test]
fn test_join_same_challenge_twice_fails() {
    let (mut svm, user) = setup();
    let profile = initialize_user(&mut svm, &user);
    let challenge_pda = create_challenge(&mut svm, &user, 0, single_req(0, 10), 0, FUTURE_DEADLINE);
    let enrollment_pda = derive_enrollment(&challenge_pda, &user.pubkey());

    let ix = join_challenge_ix(profile, challenge_pda, enrollment_pda, user.pubkey());
    assert!(send(&mut svm, ix, &user));

    let ix2 = join_challenge_ix(profile, challenge_pda, enrollment_pda, user.pubkey());
    assert!(
        !send(&mut svm, ix2, &user),
        "joining the same challenge twice must fail — init constraint blocks re-init"
    );
}

#[test]
fn test_join_without_user_profile_fails() {
    let (mut svm, user) = setup();
    let creator = new_funded_user(&mut svm);
    initialize_user(&mut svm, &creator);

    let challenge_pda = create_challenge(&mut svm, &creator, 0, single_req(0, 10), 0, FUTURE_DEADLINE);
    let fake_profile = derive_user_profile(&user.pubkey());
    let enrollment_pda = derive_enrollment(&challenge_pda, &user.pubkey());

    let ix = join_challenge_ix(fake_profile, challenge_pda, enrollment_pda, user.pubkey());
    assert!(
        !send(&mut svm, ix, &user),
        "joining without an initialised UserProfile must fail"
    );
}
