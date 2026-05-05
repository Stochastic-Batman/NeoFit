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
    send_ixs(svm, &[ix], signer)
}

fn send_ixs(svm: &mut LiteSVM, ixs: &[Instruction], signer: &Keypair) -> bool {
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&signer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[signer]).unwrap();
    match svm.send_transaction(tx) {
        Ok(_) => true,
        Err(e) => { println!("Transaction Failed: {:?}", e); false }
    }
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
    assert!(send(svm, ix, user), "initialize_user must succeed");
    pda
}

fn create_challenge(
    svm: &mut LiteSVM,
    creator: &Keypair,
    nonce: u64,
    requirements: Vec<ExerciseRequirement>,
    entry_fee: u64,
) -> Pubkey {
    let pda = derive_challenge(&creator.pubkey(), nonce);
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::CreateChallenge {
            title: "Reward Test".to_string(),
            requirements,
            entry_fee,
            deadline: FUTURE_DEADLINE,
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
    assert!(send(svm, ix, creator), "create_challenge must succeed");
    pda
}

fn join_challenge(svm: &mut LiteSVM, user: &Keypair, challenge_pda: Pubkey) -> Pubkey {
    let profile = derive_user_profile(&user.pubkey());
    let enrollment = derive_enrollment(&challenge_pda, &user.pubkey());

    let raw = svm.get_account(&challenge_pda).expect("Challenge must exist");
    let challenge = Challenge::try_deserialize(&mut raw.data.as_ref()).unwrap();

    let mut ixs = Vec::new();

    ixs.push(Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::JoinChallenge {}.data(),
        neofit::accounts::JoinChallenge {
            enrollment,
            challenge: challenge_pda,
            user_profile: profile,
            authority: user.pubkey(),
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    ));

    if challenge.entry_fee_lamports > 0 {
        ixs.push(anchor_lang::solana_program::system_instruction::transfer(
            &user.pubkey(),
            &challenge_pda,
            challenge.entry_fee_lamports,
        ));
    }

    assert!(send_ixs(svm, &ixs, user), "join_challenge must succeed");
    enrollment
}

fn log_reps_with_enrollment(
    svm: &mut LiteSVM,
    user: &Keypair,
    challenge_pda: Pubkey,
    enrollment_pda: Pubkey,
    exercise_id: u8,
    count: u32,
) {
    let profile = derive_user_profile(&user.pubkey());
    let ix = Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::LogReps { exercise_id, count }.data(),
        neofit::accounts::LogReps {
            user_profile: profile,
            enrollment: Some(enrollment_pda),
            challenge: Some(challenge_pda),
            authority: user.pubkey(),
        }
        .to_account_metas(None),
    );
    assert!(send(svm, ix, user), "log_reps must succeed");
}

fn claim_reward_ix(enrollment: Pubkey, challenge: Pubkey, authority: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::ClaimReward {}.data(),
        neofit::accounts::ClaimReward {
            enrollment,
            challenge,
            authority,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

fn get_enrollment(svm: &LiteSVM, pda: &Pubkey) -> Enrollment {
    let raw = svm.get_account(pda).expect("Enrollment must exist");
    Enrollment::try_deserialize(&mut raw.data.as_ref()).unwrap()
}

fn lamports(svm: &LiteSVM, pubkey: &Pubkey) -> u64 {
    svm.get_account(pubkey).map(|a| a.lamports).unwrap_or(0)
}

fn setup_completed_enrollment(
    svm: &mut LiteSVM,
    user: &Keypair,
    challenge_pda: Pubkey,
    requirements: &[ExerciseRequirement],
) -> Pubkey {
    let enrollment = join_challenge(svm, user, challenge_pda);
    for req in requirements {
        log_reps_with_enrollment(svm, user, challenge_pda, enrollment, req.exercise_id, req.rep_target as u32);
    }
    enrollment
}

#[test]
fn test_claim_reward_sets_reward_claimed() {
    let (mut svm, creator) = setup();
    initialize_user(&mut svm, &creator);
    let reqs = vec![ExerciseRequirement { exercise_id: 0, rep_target: 5 }];
    let challenge_pda = create_challenge(&mut svm, &creator, 0, reqs.clone(), 0);

    let user = new_funded_user(&mut svm);
    initialize_user(&mut svm, &user);
    let enrollment = setup_completed_enrollment(&mut svm, &user, challenge_pda, &reqs);

    let ix = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
    assert!(send(&mut svm, ix, &user));

    let e = get_enrollment(&svm, &enrollment);
    assert!(e.reward_claimed);
    assert!(e.completed);
}

#[test]
fn test_claim_reward_double_claim_fails() {
    let (mut svm, creator) = setup();
    initialize_user(&mut svm, &creator);
    let reqs = vec![ExerciseRequirement { exercise_id: 0, rep_target: 5 }];
    let challenge_pda = create_challenge(&mut svm, &creator, 0, reqs.clone(), 0);

    let user = new_funded_user(&mut svm);
    initialize_user(&mut svm, &user);
    let enrollment = setup_completed_enrollment(&mut svm, &user, challenge_pda, &reqs);

    let ix1 = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
    assert!(send(&mut svm, ix1, &user));

    let ix2 = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
    assert!(
        !send(&mut svm, ix2, &user),
        "double claim must be rejected"
    );
}

#[test]
fn test_claim_reward_incomplete_enrollment_fails() {
    let (mut svm, creator) = setup();
    initialize_user(&mut svm, &creator);
    let reqs = vec![ExerciseRequirement { exercise_id: 0, rep_target: 50 }];
    let challenge_pda = create_challenge(&mut svm, &creator, 0, reqs.clone(), 0);

    let user = new_funded_user(&mut svm);
    initialize_user(&mut svm, &user);
    let enrollment = join_challenge(&mut svm, &user, challenge_pda);

    log_reps_with_enrollment(&mut svm, &user, challenge_pda, enrollment, 0, 10);

    let e = get_enrollment(&svm, &enrollment);
    assert!(!e.completed, "enrollment must not be completed yet");

    let ix = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
    assert!(
        !send(&mut svm, ix, &user),
        "claiming an incomplete enrollment must fail"
    );
}

#[test]
fn test_claim_reward_payout_single_completer() {
    let (mut svm, creator) = setup();
    initialize_user(&mut svm, &creator);

    let user = new_funded_user(&mut svm);
    initialize_user(&mut svm, &user);
    
    let entry_fee = 1_000_000_000u64;
    let reqs = vec![ExerciseRequirement { exercise_id: 0, rep_target: 5 }];
    let challenge_pda = create_challenge(&mut svm, &creator, 0, reqs.clone(), entry_fee);

    let enrollment = setup_completed_enrollment(&mut svm, &user, challenge_pda, &reqs);

    let user_lamports_before = lamports(&svm, &user.pubkey());
    let challenge_lamports_before = lamports(&svm, &challenge_pda);

    let ix = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
    assert!(send(&mut svm, ix, &user));

    // pool = 1 SOL; net = 0.9 SOL (10% protocol fee); 1 completer gets 0.9 SOL
    let expected_share = entry_fee * 9_000 / 10_000;
    let tx_fee = 5_000u64;
    let user_lamports_after = lamports(&svm, &user.pubkey());
    let challenge_lamports_after = lamports(&svm, &challenge_pda);

    assert_eq!(
        user_lamports_after,
        user_lamports_before + expected_share - tx_fee,
        "user must receive net pool minus protocol fee"
    );
    assert_eq!(
        challenge_lamports_after,
        challenge_lamports_before - expected_share,
        "challenge account must lose exactly the user share"
    );
}

#[test]
fn test_claim_reward_payout_splits_among_completers() {
    let (mut svm, creator) = setup();
    initialize_user(&mut svm, &creator);

    let entry_fee = 1_000_000_000u64;
    let reqs = vec![ExerciseRequirement { exercise_id: 0, rep_target: 5 }];

    let users: Vec<Keypair> = (0..3).map(|_| {
        let u = new_funded_user(&mut svm);
        initialize_user(&mut svm, &u);
        u
    }).collect();

    let challenge_pda = create_challenge(&mut svm, &creator, 0, reqs.clone(), entry_fee);
    let expected_per_user = entry_fee * 3 * 9_000 / 10_000 / 3;

    for user in &users {
        let enrollment = setup_completed_enrollment(&mut svm, user, challenge_pda, &reqs);
        let before = lamports(&svm, &user.pubkey());
        let ix = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
        assert!(send(&mut svm, ix, user));
        let after = lamports(&svm, &user.pubkey());
        assert_eq!(after, before + expected_per_user - 5_000, "each completer must receive an equal share");
    }
}

#[test]
fn test_claim_reward_wrong_authority_fails() {
    let (mut svm, creator) = setup();
    initialize_user(&mut svm, &creator);
    let reqs = vec![ExerciseRequirement { exercise_id: 0, rep_target: 5 }];
    let challenge_pda = create_challenge(&mut svm, &creator, 0, reqs.clone(), 0);

    let user = new_funded_user(&mut svm);
    initialize_user(&mut svm, &user);
    let enrollment = setup_completed_enrollment(&mut svm, &user, challenge_pda, &reqs);

    let thief = new_funded_user(&mut svm);
    let ix = claim_reward_ix(enrollment, challenge_pda, thief.pubkey());
    assert!(
        !send(&mut svm, ix, &thief),
        "a different wallet must not be able to claim another user's reward"
    );
}

#[test]
fn test_claim_reward_multi_requirement_challenge() {
    let (mut svm, creator) = setup();
    initialize_user(&mut svm, &creator);
    let reqs = vec![
        ExerciseRequirement { exercise_id: 0, rep_target: 10 },
        ExerciseRequirement { exercise_id: 1, rep_target: 20 },
        ExerciseRequirement { exercise_id: 4, rep_target: 15 },
    ];
    let challenge_pda = create_challenge(&mut svm, &creator, 0, reqs.clone(), 0);

    let user = new_funded_user(&mut svm);
    initialize_user(&mut svm, &user);
    let enrollment = join_challenge(&mut svm, &user, challenge_pda);

    log_reps_with_enrollment(&mut svm, &user, challenge_pda, enrollment, 0, 10);
    log_reps_with_enrollment(&mut svm, &user, challenge_pda, enrollment, 1, 20);

    let e = get_enrollment(&svm, &enrollment);
    assert!(!e.completed, "must not complete until all requirements are met");

    let ix = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
    assert!(!send(&mut svm, ix, &user), "partial completion must not allow claiming");

    svm.expire_blockhash();

    log_reps_with_enrollment(&mut svm, &user, challenge_pda, enrollment, 4, 15);

    let e = get_enrollment(&svm, &enrollment);
    assert!(e.completed);

    let ix = claim_reward_ix(enrollment, challenge_pda, user.pubkey());
    assert!(send(&mut svm, ix, &user));
}
