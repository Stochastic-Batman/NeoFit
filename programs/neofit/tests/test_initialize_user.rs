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

fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/neofit.so");
    svm.add_program(neofit::id(), bytes).unwrap();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();
    (svm, payer)
}

fn derive_user_profile(authority: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"user_profile", authority.as_ref()],
        &neofit::id(),
    )
}

fn initialize_user_ix(user_profile: Pubkey, authority: Pubkey) -> Instruction {
    Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::InitializeUser {}.data(),
        neofit::accounts::InitializeUser {
            user_profile,
            authority,
            system_program: anchor_lang::solana_program::system_program::ID,
        }
        .to_account_metas(None),
    )
}

#[test]
fn test_initialize_user_creates_account() {
    let (mut svm, payer) = setup();
    let (pda, _bump) = derive_user_profile(&payer.pubkey());

    let ix = initialize_user_ix(pda, payer.pubkey());
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    assert!(svm.send_transaction(tx).is_ok());

    let raw = svm.get_account(&pda).expect("UserProfile account must exist after init");
    let profile = UserProfile::try_deserialize(&mut raw.data.as_ref())
        .expect("must deserialize as UserProfile");

    assert_eq!(profile.authority, payer.pubkey());
    assert_eq!(profile.total_reps, 0);
    assert_eq!(profile.streak_days, 0);
    assert_eq!(profile.last_workout_ts, 0);
    assert!(profile.rep_counts.is_empty());
    assert_eq!(profile.bump, _bump);
}

#[test]
fn test_initialize_user_default_username_format() {
    let (mut svm, payer) = setup();
    let (pda, _bump) = derive_user_profile(&payer.pubkey());

    let ix = initialize_user_ix(pda, payer.pubkey());
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    svm.send_transaction(tx).unwrap();

    let raw = svm.get_account(&pda).unwrap();
    let profile = UserProfile::try_deserialize(&mut raw.data.as_ref()).unwrap();

    let addr = payer.pubkey().to_string();
    let expected = format!("{}default_as_irl{}", &addr[..4], &addr[addr.len() - 4..]);

    assert_eq!(profile.username, expected);
    assert!(profile.username.len() <= 22);
}

#[test]
fn test_initialize_user_twice_fails() {
    let (mut svm, payer) = setup();
    let (pda, _bump) = derive_user_profile(&payer.pubkey());

    for _ in 0..2 {
        let ix = initialize_user_ix(pda, payer.pubkey());
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
        let tx =
            VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
        let _ = svm.send_transaction(tx);
    }

    let ix = initialize_user_ix(pda, payer.pubkey());
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();

    assert!(
        svm.send_transaction(tx).is_err(),
        "second initialize must fail — init constraint forbids re-init"
    );
}

#[test]
fn test_initialize_user_different_wallets_get_different_pdas() {
    let (mut svm, alice) = setup();
    let bob = Keypair::new();
    svm.airdrop(&bob.pubkey(), 10_000_000_000).unwrap();

    let (alice_pda, _) = derive_user_profile(&alice.pubkey());
    let (bob_pda, _) = derive_user_profile(&bob.pubkey());
    assert_ne!(alice_pda, bob_pda);

    for (keypair, pda) in [(&alice, alice_pda), (&bob, bob_pda)] {
        let ix = initialize_user_ix(pda, keypair.pubkey());
        let blockhash = svm.latest_blockhash();
        let msg =
            Message::new_with_blockhash(&[ix], Some(&keypair.pubkey()), &blockhash);
        let tx =
            VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[keypair]).unwrap();
        assert!(svm.send_transaction(tx).is_ok());
    }

    let alice_raw = svm.get_account(&alice_pda).unwrap();
    let alice_profile = UserProfile::try_deserialize(&mut alice_raw.data.as_ref()).unwrap();
    assert_eq!(alice_profile.authority, alice.pubkey());

    let bob_raw = svm.get_account(&bob_pda).unwrap();
    let bob_profile = UserProfile::try_deserialize(&mut bob_raw.data.as_ref()).unwrap();
    assert_eq!(bob_profile.authority, bob.pubkey());
}
