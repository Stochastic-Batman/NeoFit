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

fn initialize_user(svm: &mut LiteSVM, payer: &Keypair) -> Pubkey {
    let (pda, _) = derive_user_profile(&payer.pubkey());
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

fn update_username_ix(user_profile: Pubkey, authority: Pubkey, new_username: &str) -> Instruction {
    Instruction::new_with_bytes(
        neofit::id(),
        &neofit::instruction::UpdateUsername {
            new_username: new_username.to_string(),
        }
        .data(),
        neofit::accounts::UpdateUsername {
            user_profile,
            authority,
        }
        .to_account_metas(None),
    )
}

#[test]
fn test_update_username_ok() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    let ix = update_username_ix(pda, payer.pubkey(), "neofit_legend");
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let raw = svm.get_account(&pda).unwrap();
    let profile = UserProfile::try_deserialize(&mut raw.data.as_ref()).unwrap();
    assert_eq!(profile.username, "neofit_legend");
}

#[test]
fn test_update_username_max_length_accepted() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    let exactly_22 = "a".repeat(22);
    let ix = update_username_ix(pda, payer.pubkey(), &exactly_22);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(svm.send_transaction(tx).is_ok());

    let raw = svm.get_account(&pda).unwrap();
    let profile = UserProfile::try_deserialize(&mut raw.data.as_ref()).unwrap();
    assert_eq!(profile.username, exactly_22);
}

#[test]
fn test_update_username_too_long_fails() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    let too_long = "a".repeat(23);
    let ix = update_username_ix(pda, payer.pubkey(), &too_long);
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "username exceeding MAX_USERNAME_LEN must be rejected"
    );
}

#[test]
fn test_update_username_empty_fails() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    let ix = update_username_ix(pda, payer.pubkey(), "");
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "empty username must be rejected"
    );
}

#[test]
fn test_update_username_wrong_authority_fails() {
    let (mut svm, alice) = setup();
    let pda = initialize_user(&mut svm, &alice);

    let eve = Keypair::new();
    svm.airdrop(&eve.pubkey(), 10_000_000_000).unwrap();

    let ix = update_username_ix(pda, eve.pubkey(), "hacker");
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[ix], Some(&eve.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&eve]).unwrap();
    assert!(
        svm.send_transaction(tx).is_err(),
        "a different wallet must not be able to update another user's username"
    );
}

#[test]
fn test_update_username_can_be_called_multiple_times() {
    let (mut svm, payer) = setup();
    let pda = initialize_user(&mut svm, &payer);

    for name in ["first_handle", "second_handle", "final_form"] {
        let ix = update_username_ix(pda, payer.pubkey(), name);
        let blockhash = svm.latest_blockhash();
        let msg = Message::new_with_blockhash(&[ix], Some(&payer.pubkey()), &blockhash);
        let tx =
            VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&payer]).unwrap();
        assert!(svm.send_transaction(tx).is_ok());
    }

    let raw = svm.get_account(&pda).unwrap();
    let profile = UserProfile::try_deserialize(&mut raw.data.as_ref()).unwrap();
    assert_eq!(profile.username, "final_form");
}
