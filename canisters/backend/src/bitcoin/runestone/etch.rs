use crate::bitcoin_lib::{
    Address, Amount, EcdsaSighashType, Network, OutPoint, PublicKey, Script, ScriptBuf, Sequence,
    TapLeafHash, TapSighashType, Transaction, TxIn, TxOut, Witness, XOnlyPublicKey,
    absolute::LockTime,
    hashes::Hash,
    key::{Secp256k1, constants::SCHNORR_SIGNATURE_SIZE},
    opcodes,
    script::{Builder, PushBytesBuf},
    secp256k1::schnorr,
    sighash::{Prevouts, SighashCache},
    taproot::{self, ControlBlock, LeafVersion, TaprootBuilder},
    transaction::Version,
};
use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Utxo};
use icrc_ledger_types::icrc1::account::Account;
use ordinals::{Artifact, Etching, Runestone, SpacedRune};

use crate::{
    bitcoin::{
        DUST_THRESHOLD,
        signer::{ecdsa::ecdsa_sign, schnorr::schnorr_sign},
        utils::{account_to_derivation_path, derive_public_key, sec1_to_der, slice_to_txid},
    },
    state::{read_config, write_utxo_manager},
    txn_handler::TransactionType,
};

use super::{MAX_STANDARD_OP_RETURN_SIZE, TARGET_POSTAGE, inscription::Inscription};

fn build_reveal_transaction(
    commit_input_index: usize,
    control_block: &ControlBlock,
    fee_per_vbytes: u64,
    output: Vec<TxOut>,
    input: Vec<OutPoint>,
    script: &Script,
) -> (Transaction, Amount) {
    for i in input.iter() {
        ic_cdk::println!("{}, {}", i.txid, i.vout);
    }
    let reveal_txn = Transaction {
        input: input
            .into_iter()
            .map(|previous_output| TxIn {
                previous_output,
                sequence: Sequence::from_height(Runestone::COMMIT_CONFIRMATIONS - 1),
                script_sig: ScriptBuf::new(),
                witness: Witness::new(),
            })
            .collect(),
        output,
        lock_time: LockTime::ZERO,
        version: Version(2),
    };
    let fee = {
        let mut reveal_txn = reveal_txn.clone();

        for (index, txin) in reveal_txn.input.iter_mut().enumerate() {
            if index == commit_input_index {
                txin.witness.push(
                    taproot::Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
                        .unwrap()
                        .to_vec(),
                );
                txin.witness.push(script);
                txin.witness.push(control_block.serialize());
            } else {
                txin.witness = Witness::from_slice(&[&[0; SCHNORR_SIGNATURE_SIZE]]);
            }
        }

        let vsize = reveal_txn.vsize() as u64;
        Amount::from_sat((vsize * fee_per_vbytes) / 1000)
    };

    (reveal_txn, fee)
}

pub struct EtchingArgs {
    pub reveal_address: Address,
    pub logo: Option<Vec<u8>>,
    pub content_type: Option<Vec<u8>>,
    pub spaced_rune: SpacedRune,
    pub premine: u128,
    pub divisibility: u8,
    pub symbol: Option<char>,
    pub turbo: bool,
    pub fee_payer: Address,
    pub fee_payer_account: Account,
    pub fee_per_vbytes: u64,
}

// NOTE:
// this function will return a signed reveal transaction
// only commit transaction is needed to by signed!
pub async fn etch(
    EtchingArgs {
        reveal_address,
        logo,
        content_type,
        spaced_rune,
        premine,
        divisibility,
        symbol,
        turbo,
        fee_payer,
        fee_payer_account,
        fee_per_vbytes,
    }: EtchingArgs,
) -> Result<(TransactionType, (String, String)), u64> {
    let SpacedRune { rune, spacers } = spaced_rune;
    let inscription = Inscription::new(logo, content_type, rune);

    let (mut reveal_input, mut reveal_output) = (vec![OutPoint::null()], vec![]);

    let etching = Etching {
        divisibility: Some(divisibility),
        premine: Some(premine),
        rune: Some(rune),
        spacers: Some(spacers),
        symbol,
        turbo,
        terms: None, // this will make rune unmintable
    };

    if premine > 0 {
        // let output = reveal_output.len() as u32;
        reveal_output.push(TxOut {
            script_pubkey: reveal_address.script_pubkey(),
            value: TARGET_POSTAGE,
        });
    }

    let runestone = Runestone {
        edicts: vec![],
        etching: Some(etching),
        mint: None,
        pointer: (premine > 0).then_some(reveal_output.len() as u32 - 1),
    };

    let enciphered = runestone.encipher();
    if enciphered.len() > MAX_STANDARD_OP_RETURN_SIZE {
        ic_cdk::trap("runestone greater than maximum OP_RETURN size");
    }

    reveal_output.push(TxOut {
        value: Amount::ZERO,
        script_pubkey: enciphered.clone(),
    });

    let (schnorr_public_key, network) = read_config(|config| {
        let schnorr_public_key = config.schnorr_public_key().public_key;
        let network = match config.bitcoin_network() {
            BitcoinNetwork::Mainnet => Network::Bitcoin,
            BitcoinNetwork::Testnet => Network::Testnet,
            BitcoinNetwork::Regtest => Network::Regtest,
        };
        (schnorr_public_key, network)
    });
    let secp256k1 = Secp256k1::new();
    let schnorr_public_key: XOnlyPublicKey =
        PublicKey::from_slice(&schnorr_public_key).unwrap().into();
    let reveal_script = Builder::new()
        .push_slice(schnorr_public_key.serialize())
        .push_opcode(opcodes::all::OP_CHECKSIG);
    let reveal_script = inscription
        .append_reveal_script_to_builder(reveal_script)
        .into_script();

    let taproot_spend_info = TaprootBuilder::new()
        .add_leaf(0, reveal_script.clone())
        .expect("adding leaf should work")
        .finalize(&secp256k1, schnorr_public_key)
        .expect("finalizing taproot builder should work");

    let control_block = taproot_spend_info
        .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
        .expect("should compute control block");

    let commit_tx_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), network);

    let commit_input_index = 0;

    let (_, reveal_fee) = build_reveal_transaction(
        commit_input_index,
        &control_block,
        fee_per_vbytes,
        reveal_output.clone(),
        reveal_input.clone(),
        &reveal_script,
    );

    let mut target_value = reveal_fee;
    // for premining
    target_value += TARGET_POSTAGE;

    let (mut commit_txn, utxos) = build_commit_transaction_with_fee(
        &fee_payer,
        commit_tx_address.script_pubkey(),
        fee_per_vbytes,
        target_value,
    )?;

    let (vout, _) = commit_txn
        .output
        .iter()
        .enumerate()
        .find(|(_, output)| output.script_pubkey == commit_tx_address.script_pubkey())
        .expect("should find the sat commit output");

    let txn_bytes = bitcoin::consensus::serialize(&commit_txn);
    ic_cdk::println!("commit txn before signing");
    ic_cdk::println!("{}", hex::encode(txn_bytes));

    let (path, pubkey) = read_config(|config| {
        let ecdsa_pubkey = config.ecdsa_public_key();
        let path = account_to_derivation_path(&fee_payer_account);
        let pubkey = derive_public_key(&ecdsa_pubkey, &path).public_key;
        (
            path.iter().map(|x| x.to_vec()).collect::<Vec<Vec<u8>>>(),
            pubkey,
        )
    });

    let txn_cache = SighashCache::new(commit_txn.clone());

    for (index, input) in commit_txn.input.iter_mut().enumerate() {
        let sighash = txn_cache
            .legacy_signature_hash(
                index,
                &fee_payer.script_pubkey(),
                EcdsaSighashType::All.to_u32(),
            )
            .unwrap();

        let signature = ecdsa_sign(sighash.as_byte_array().to_vec(), path.clone())
            .await
            .signature;
        let mut signature = sec1_to_der(signature);
        signature.push(EcdsaSighashType::All.to_u32() as u8);
        let signature = PushBytesBuf::try_from(signature).unwrap();
        let pubkey = PushBytesBuf::try_from(pubkey.clone()).unwrap();
        input.script_sig = Builder::new()
            .push_slice(signature)
            .push_slice(pubkey)
            .into_script();
        input.witness.clear();
    }

    reveal_input[commit_input_index] = OutPoint {
        txid: commit_txn.compute_txid(),
        vout: vout.try_into().unwrap(),
    };

    let (mut reveal_txn, _) = build_reveal_transaction(
        commit_input_index,
        &control_block,
        fee_per_vbytes,
        reveal_output,
        reveal_input,
        &reveal_script,
    );

    for output in reveal_txn.output.iter() {
        if output.value < output.script_pubkey.minimal_non_dust() {
            ic_cdk::trap("Commit transaction will be dust");
        }
    }

    let mut sighashcache = SighashCache::new(&mut reveal_txn);
    let sighash = sighashcache
        .taproot_script_spend_signature_hash(
            commit_input_index,
            &Prevouts::All(&[commit_txn.output[vout].clone()]),
            TapLeafHash::from_script(&reveal_script, LeafVersion::TapScript),
            TapSighashType::Default,
        )
        .expect("signature hash should compute");

    let sig = schnorr_sign(sighash.as_byte_array().to_vec(), vec![])
        .await
        .signature;

    let witness = sighashcache
        .witness_mut(commit_input_index)
        .expect("getting mutable reference should work");
    witness.push(
        taproot::Signature {
            signature: schnorr::Signature::from_slice(sig.as_slice())
                .expect("should parse signature"),
            sighash_type: TapSighashType::Default,
        }
        .to_vec(),
    );

    witness.push(reveal_script);
    witness.push(control_block.serialize());

    if Runestone::decipher(&reveal_txn).unwrap() != Artifact::Runestone(runestone) {
        ic_cdk::trap("Transaction doesn't contain runestone")
    }
    let commit_txid = commit_txn.compute_txid().to_string();
    let reveal_txid = reveal_txn.compute_txid().to_string();
    Ok((
        TransactionType::Etching {
            commit_tx_address,
            commit: commit_txn,
            reveal: reveal_txn,
            fee_utxos: utxos,
            fee_payer,
        },
        (commit_txid, reveal_txid),
    ))
}

fn build_commit_transaction_with_fee(
    fee_payer: &Address,
    recipient: ScriptBuf,
    fee_per_vbytes: u64,
    target: Amount,
) -> Result<(Transaction, Vec<Utxo>), u64> {
    let payer = &fee_payer.to_string();
    if !recipient.is_op_return() {
        let dust_value = recipient.minimal_non_dust();

        if target < dust_value {
            ic_cdk::trap("DUST VALUE")
        }
    }
    let mut total_fee = 0;
    loop {
        let (mut input, mut output) = (vec![], vec![]);

        let (utxos_to_spend, total_spent) = write_utxo_manager(|manager| {
            let mut utxos = vec![];
            let mut total_spent = 0;
            while let Some(utxo) = manager.get_bitcoin_utxo(payer) {
                total_spent += utxo.value;
                utxos.push(utxo);
                if total_spent >= target.to_sat() + total_fee {
                    break;
                }
            }
            if total_spent < target.to_sat() + total_fee {
                manager.record_bitcoin_utxos(payer, utxos);
                return Err(target.to_sat() + total_fee);
            }
            Ok((utxos, total_spent))
        })?;

        utxos_to_spend.iter().for_each(|utxo| {
            let txin = TxIn {
                previous_output: OutPoint {
                    txid: slice_to_txid(&utxo.outpoint.txid),
                    vout: utxo.outpoint.vout,
                },
                script_sig: ScriptBuf::new(),
                sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
                witness: Witness::from_slice(&[&[0; SCHNORR_SIGNATURE_SIZE]]),
            };
            input.push(txin);
        });

        ic_cdk::println!("{}", total_spent);

        output.push(TxOut {
            value: target,
            script_pubkey: recipient.clone(),
        });
        let remaining = total_spent - target.to_sat() - total_fee;
        if remaining >= DUST_THRESHOLD {
            output.push(TxOut {
                value: Amount::from_sat(remaining),
                script_pubkey: fee_payer.script_pubkey(),
            });
        }

        let txn = Transaction {
            input,
            output,
            version: Version(2),
            lock_time: LockTime::ZERO,
        };

        let txn_vsize = txn.vsize() as u64;

        if (txn_vsize * fee_per_vbytes) / 1000 == total_fee {
            return Ok((txn, utxos_to_spend));
        } else {
            write_utxo_manager(|manager| {
                manager.record_bitcoin_utxos(payer, utxos_to_spend);
            });
            total_fee = (txn_vsize * fee_per_vbytes) / 1000;
        }
    }
}
