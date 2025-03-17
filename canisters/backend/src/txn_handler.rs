use std::time::Duration;

use crate::bitcoin_lib::{
    Address, Amount, EcdsaSighashType, OutPoint, ScriptBuf, Sequence, Transaction, TxIn, TxOut,
    Witness,
    absolute::LockTime,
    hashes::Hash,
    script::{Builder, PushBytesBuf},
    sighash::SighashCache,
    transaction::Version,
};
use candid::CandidType;
use ic_cdk::api::management_canister::bitcoin::{
    GetUtxosRequest, SendTransactionRequest, Utxo, bitcoin_get_utxos, bitcoin_send_transaction,
};
use icrc_ledger_types::icrc1::account::Account;
use ordinals::{Edict, Runestone};
use slotmap::Key;

use crate::{
    bitcoin::{DUST_THRESHOLD, signer::ecdsa::ecdsa_sign, utils::*},
    indexer::RuneId,
    state::{
        queue::ScheduledTransaction, read_config, utxo_manager::RunicUtxo, write_scheduled_state,
        write_utxo_manager,
    },
};

#[derive(CandidType, PartialEq, Eq)]
pub enum SubmittedTxidType {
    Bitcoin { txid: String },
}

pub enum TransactionType {
    Etching {
        commit_tx_address: Address,
        commit: Transaction,
        reveal: Transaction,
        fee_utxos: Vec<Utxo>,
        fee_payer: Address,
    },
    Bitcoin {
        utxos: Vec<Utxo>,
        txn: Transaction,
        sender: Address,
        sender_account: Account,
    },
    Rune {
        runic_utxos: Vec<RunicUtxo>,
        runeid: RuneId,
        rune_amount: u128,
        rune_sender: Box<Address>,
        rune_receiver: Box<Address>,
        rune_sender_account: Account,
        fee_utxos: Vec<Utxo>,
        fee: u64,
        fee_payer: Box<Address>,
        fee_payer_account: Account,
        postage: Amount,
    },
    Combined {
        runic_utxos: Vec<RunicUtxo>,
        runeid: RuneId,
        rune_amount: u128,
        rune_sender: Box<Address>,
        rune_receiver: Box<Address>,
        rune_sender_account: Account,
        bitcoin_utxos: Vec<Utxo>,
        bitcoin_amount: u64,
        bitcoin_sender: Box<Address>,
        bitcoin_receiver: Box<Address>,
        bitcoin_sender_account: Account,
        fee_utxos: Vec<Utxo>,
        fee: u64,
        fee_payer: Box<Address>,
        fee_payer_account: Account,
        postage: Amount,
    },
}

impl TransactionType {
    pub async fn submit(self) -> SubmittedTxidType {
        match self {
            Self::Etching {
                commit_tx_address,
                commit,
                reveal,
                fee_utxos,
                fee_payer,
            } => {
                let (network, timer) = read_config(|config| {
                    (
                        config.bitcoin_network(),
                        config.get_timer_for_txn_submission(),
                    )
                });
                let txid = commit.compute_txid().to_string();
                let txn = bitcoin::consensus::serialize(&commit);
                ic_cdk::println!("commit: {}", hex::encode(&txn));
                if bitcoin_send_transaction(SendTransactionRequest {
                    transaction: txn,
                    network,
                })
                .await
                .is_err()
                {
                    write_utxo_manager(|manager| {
                        manager.record_bitcoin_utxos(fee_payer.to_string().as_ref(), fee_utxos);
                    });
                    ic_cdk::trap("failed submitting the transaction")
                }
                write_scheduled_state(|state| {
                    let id = state.get_id();
                    let timer_id =
                        ic_cdk_timers::set_timer_interval(Duration::from_secs(timer), move || {
                            ic_cdk::spawn(submit_txn(id))
                        });
                    state.record_txn(
                        id,
                        ScheduledTransaction {
                            txn: reveal,
                            commit_tx_address: commit_tx_address.to_string(),
                            timer_id: timer_id.data(),
                        },
                    );
                });
                SubmittedTxidType::Bitcoin { txid }
            }
            Self::Bitcoin {
                utxos,
                txn,
                sender,
                sender_account,
            } => {
                let mut txn: Transaction = txn;
                let (path, pubkey) = read_config(|config| {
                    let ecdsa_pubkey = config.ecdsa_public_key();
                    let path = account_to_derivation_path(&sender_account);
                    let pubkey = derive_public_key(&ecdsa_pubkey, &path).public_key;
                    (
                        path.iter().map(|x| x.to_vec()).collect::<Vec<Vec<u8>>>(),
                        pubkey,
                    )
                });

                let txn_cache = SighashCache::new(txn.clone());
                let network = read_config(|config| config.bitcoin_network());

                for (index, input) in txn.input.iter_mut().enumerate() {
                    let sighash = txn_cache
                        .legacy_signature_hash(
                            index,
                            &sender.script_pubkey(),
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
                let txid = txn.compute_txid().to_string();
                let txn_bytes = bitcoin::consensus::serialize(&txn);
                ic_cdk::println!("bitcoin transaction bytes:");
                ic_cdk::println!("{}", hex::encode(&txn_bytes));
                if bitcoin_send_transaction(SendTransactionRequest {
                    transaction: txn_bytes,
                    network,
                })
                .await
                .is_err()
                {
                    write_utxo_manager(|manager| {
                        manager.record_bitcoin_utxos(sender.to_string().as_ref(), utxos);
                    });
                    ic_cdk::trap("failed submitting the transaction")
                }
                SubmittedTxidType::Bitcoin { txid }
            }

            Self::Rune {
                runic_utxos,
                runeid,
                rune_amount,
                rune_sender,
                rune_receiver,
                rune_sender_account,
                fee_utxos,
                fee,
                fee_payer,
                fee_payer_account,
                postage,
            } => {
                let mut input = vec![];
                let mut output = vec![];

                let mut runic_total_spent = 0;
                let mut bitcoin_spent_in_runic = 0;
                let mut fee_total_spent = 0;

                let mut indexes_for_sender = vec![];

                runic_utxos.iter().for_each(|RunicUtxo { utxo, balance }| {
                    runic_total_spent += balance;
                    bitcoin_spent_in_runic += utxo.value;
                    let txin = TxIn {
                        sequence: Sequence::MAX,
                        witness: Witness::new(),
                        script_sig: ScriptBuf::new(),
                        previous_output: OutPoint {
                            txid: slice_to_txid(&utxo.outpoint.txid),
                            vout: utxo.outpoint.vout,
                        },
                    };
                    let len = input.len();
                    input.push(txin);
                    indexes_for_sender.push(len);
                });

                let need_change_rune_output =
                    runic_total_spent > rune_amount || runic_utxos.len() > 1;

                let required_bitcoin_for_postage = if need_change_rune_output {
                    postage.to_sat() * 2
                } else {
                    postage.to_sat()
                };

                if need_change_rune_output {
                    let id = ordinals::RuneId {
                        block: runeid.block,
                        tx: runeid.tx,
                    };
                    let rune = Runestone {
                        edicts: vec![Edict {
                            id,
                            amount: rune_amount,
                            output: 2,
                        }],
                        ..Default::default()
                    };
                    output.push(TxOut {
                        script_pubkey: rune.encipher(),
                        value: Amount::from_sat(0),
                    });
                    output.push(TxOut {
                        script_pubkey: rune_sender.script_pubkey(),
                        value: postage,
                    });
                    output.push(TxOut {
                        script_pubkey: rune_receiver.script_pubkey(),
                        value: postage,
                    });
                } else {
                    output.push(TxOut {
                        script_pubkey: rune_receiver.script_pubkey(),
                        value: postage,
                    });
                }

                // fee

                fee_utxos.iter().for_each(|utxo| {
                    fee_total_spent += utxo.value;
                    let txin = TxIn {
                        sequence: Sequence::MAX,
                        witness: Witness::new(),
                        script_sig: ScriptBuf::new(),
                        previous_output: OutPoint {
                            txid: slice_to_txid(&utxo.outpoint.txid),
                            vout: utxo.outpoint.vout,
                        },
                    };
                    input.push(txin);
                });

                let remaining = fee_total_spent - fee - required_bitcoin_for_postage;
                if remaining >= DUST_THRESHOLD {
                    output.push(TxOut {
                        script_pubkey: fee_payer.script_pubkey(),
                        value: Amount::from_sat(remaining),
                    });
                }

                let mut txn = Transaction {
                    input,
                    output,
                    version: Version(2),
                    lock_time: LockTime::ZERO,
                };

                // signing the transaction

                let (rune_sender_path, rune_sender_pubkey, fee_payer_path, fee_payer_pubkey) =
                    read_config(|config| {
                        let ecdsa_public_key = config.ecdsa_public_key();
                        let rune_sender_path = account_to_derivation_path(&rune_sender_account);
                        let rune_sender_pubkey =
                            derive_public_key(&ecdsa_public_key, &rune_sender_path).public_key;
                        let fee_payer_path = account_to_derivation_path(&fee_payer_account);
                        let fee_payer_pubkey =
                            derive_public_key(&ecdsa_public_key, &fee_payer_path).public_key;
                        (
                            rune_sender_path
                                .iter()
                                .map(|x| x.to_vec())
                                .collect::<Vec<Vec<u8>>>(),
                            rune_sender_pubkey,
                            fee_payer_path
                                .iter()
                                .map(|x| x.to_vec())
                                .collect::<Vec<Vec<u8>>>(),
                            fee_payer_pubkey,
                        )
                    });

                let txn_cache = SighashCache::new(txn.clone());
                let network = read_config(|config| config.bitcoin_network());

                for (index, input) in txn.input.iter_mut().enumerate() {
                    if indexes_for_sender.contains(&index) {
                        let sighash = txn_cache
                            .legacy_signature_hash(
                                index,
                                &rune_sender.script_pubkey(),
                                EcdsaSighashType::All.to_u32(),
                            )
                            .unwrap();
                        let signature =
                            ecdsa_sign(sighash.as_byte_array().to_vec(), rune_sender_path.clone())
                                .await
                                .signature;
                        let mut signature = sec1_to_der(signature);
                        signature.push(EcdsaSighashType::All.to_u32() as u8);
                        let signature = PushBytesBuf::try_from(signature).unwrap();
                        let pubkey = PushBytesBuf::try_from(rune_sender_pubkey.clone()).unwrap();

                        input.script_sig = Builder::new()
                            .push_slice(signature)
                            .push_slice(pubkey)
                            .into_script();
                        input.witness.clear();
                    } else {
                        let sighash = txn_cache
                            .legacy_signature_hash(
                                index,
                                &fee_payer.script_pubkey(),
                                EcdsaSighashType::All.to_u32(),
                            )
                            .unwrap();
                        let signature =
                            ecdsa_sign(sighash.as_byte_array().to_vec(), fee_payer_path.clone())
                                .await
                                .signature;
                        let mut signature = sec1_to_der(signature);
                        signature.push(EcdsaSighashType::All.to_u32() as u8);
                        let signature = PushBytesBuf::try_from(signature).unwrap();
                        let pubkey = PushBytesBuf::try_from(fee_payer_pubkey.clone()).unwrap();

                        input.script_sig = Builder::new()
                            .push_slice(signature)
                            .push_slice(pubkey)
                            .into_script();
                        input.witness.clear();
                    }
                }

                let txid = txn.compute_txid().to_string();
                let txn_bytes = bitcoin::consensus::serialize(&txn);
                ic_cdk::println!("{}", hex::encode(&txn_bytes));

                bitcoin_send_transaction(SendTransactionRequest {
                    transaction: txn_bytes,
                    network,
                })
                .await
                .unwrap();

                SubmittedTxidType::Bitcoin { txid }
            }
            Self::Combined {
                runic_utxos,
                runeid,
                rune_amount,
                rune_sender,
                rune_receiver,
                rune_sender_account,
                bitcoin_utxos,
                bitcoin_amount,
                bitcoin_sender,
                bitcoin_receiver,
                bitcoin_sender_account,
                fee_utxos,
                fee,
                fee_payer,
                fee_payer_account,
                postage,
            } => {
                let mut runic_total_spent = 0;
                let mut btc_in_runic_spent = 0;
                let mut btc_total_spent = 0;
                let mut fee_total_spent = 0;

                let mut index_of_utxos_to_be_signed_by_rune_sender = vec![];
                let mut index_of_utxos_to_be_signed_by_btc_sender = vec![];
                let mut index_of_utxos_to_be_signed_by_fee_payer = vec![];

                let (mut input, mut output) = (vec![], vec![]);

                runic_utxos.iter().for_each(|RunicUtxo { utxo, balance }| {
                    runic_total_spent += balance;
                    btc_in_runic_spent += utxo.value;
                    let txin = TxIn {
                        script_sig: ScriptBuf::new(),
                        witness: Witness::new(),
                        sequence: Sequence::MAX,
                        previous_output: OutPoint {
                            txid: slice_to_txid(&utxo.outpoint.txid),
                            vout: utxo.outpoint.vout,
                        },
                    };
                    let len = input.len();
                    index_of_utxos_to_be_signed_by_rune_sender.push(len);
                    input.push(txin);
                });

                let need_change_rune_output =
                    runic_total_spent > rune_amount || runic_utxos.len() > 1;
                let required_postage_btc = if need_change_rune_output {
                    postage.to_sat() * 2
                } else {
                    postage.to_sat()
                } - btc_in_runic_spent;

                if need_change_rune_output {
                    let runestone = Runestone {
                        edicts: vec![Edict {
                            id: ordinals::RuneId {
                                block: runeid.block,
                                tx: runeid.tx,
                            },
                            amount: rune_amount,
                            output: 2,
                        }],
                        ..Default::default()
                    };
                    output.push(TxOut {
                        script_pubkey: runestone.encipher(),
                        value: Amount::from_sat(0),
                    });

                    output.push(TxOut {
                        script_pubkey: rune_sender.script_pubkey(),
                        value: postage,
                    });

                    output.push(TxOut {
                        script_pubkey: rune_receiver.script_pubkey(),
                        value: postage,
                    });
                } else {
                    output.push(TxOut {
                        script_pubkey: rune_receiver.script_pubkey(),
                        value: postage,
                    });
                }

                // btc
                bitcoin_utxos.iter().for_each(|utxo| {
                    btc_total_spent += utxo.value;
                    let txin = TxIn {
                        script_sig: ScriptBuf::new(),
                        witness: Witness::new(),
                        sequence: Sequence::MAX,
                        previous_output: OutPoint {
                            txid: slice_to_txid(&utxo.outpoint.txid),
                            vout: utxo.outpoint.vout,
                        },
                    };
                    let len = input.len();
                    index_of_utxos_to_be_signed_by_btc_sender.push(len);
                    input.push(txin);
                });

                if fee_payer == bitcoin_sender {
                    let remaining = btc_total_spent - bitcoin_amount - fee - required_postage_btc;

                    output.push(TxOut {
                        script_pubkey: bitcoin_receiver.script_pubkey(),
                        value: Amount::from_sat(bitcoin_amount),
                    });

                    if remaining > DUST_THRESHOLD {
                        output.push(TxOut {
                            script_pubkey: bitcoin_sender.script_pubkey(),
                            value: Amount::from_sat(remaining),
                        });
                    }
                } else {
                    output.push(TxOut {
                        script_pubkey: bitcoin_receiver.script_pubkey(),
                        value: Amount::from_sat(bitcoin_amount),
                    });

                    // fee

                    fee_utxos.iter().for_each(|utxo| {
                        fee_total_spent += utxo.value;
                        let txin = TxIn {
                            sequence: Sequence::MAX,
                            script_sig: ScriptBuf::new(),
                            witness: Witness::new(),
                            previous_output: OutPoint {
                                txid: slice_to_txid(&utxo.outpoint.txid),
                                vout: utxo.outpoint.vout,
                            },
                        };
                        let len = input.len();
                        index_of_utxos_to_be_signed_by_fee_payer.push(len);
                        input.push(txin);
                    });

                    let remaining = fee_total_spent - fee - required_postage_btc;
                    output.push(TxOut {
                        script_pubkey: fee_payer.script_pubkey(),
                        value: Amount::from_sat(remaining),
                    });
                }

                let mut txn = Transaction {
                    input,
                    output,
                    version: Version(2),
                    lock_time: LockTime::ZERO,
                };

                // signing

                let (
                    rune_sender_path,
                    rune_sender_pubkey,
                    bitcoin_sender_path,
                    bitcoin_sender_pubkey,
                    fee_payer_path,
                    fee_payer_pubkey,
                ) = read_config(|config| {
                    let ecdsa_public_key = config.ecdsa_public_key();
                    let rune_sender_path = account_to_derivation_path(&rune_sender_account);
                    let rune_sender_pubkey =
                        derive_public_key(&ecdsa_public_key, &rune_sender_path).public_key;
                    let bitcoin_sender_path = account_to_derivation_path(&bitcoin_sender_account);
                    let bitcoin_sender_pubkey =
                        derive_public_key(&ecdsa_public_key, &bitcoin_sender_path).public_key;
                    let fee_payer_path = account_to_derivation_path(&fee_payer_account);
                    let fee_payer_pubkey =
                        derive_public_key(&ecdsa_public_key, &fee_payer_path).public_key;
                    (
                        rune_sender_path
                            .iter()
                            .map(|x| x.to_vec())
                            .collect::<Vec<Vec<u8>>>(),
                        rune_sender_pubkey,
                        bitcoin_sender_path
                            .iter()
                            .map(|x| x.to_vec())
                            .collect::<Vec<Vec<u8>>>(),
                        bitcoin_sender_pubkey,
                        fee_payer_path
                            .iter()
                            .map(|x| x.to_vec())
                            .collect::<Vec<Vec<u8>>>(),
                        fee_payer_pubkey,
                    )
                });

                let txn_cache = SighashCache::new(txn.clone());

                let network = read_config(|config| config.bitcoin_network());

                for (index, input) in txn.input.iter_mut().enumerate() {
                    if index_of_utxos_to_be_signed_by_rune_sender.contains(&index) {
                        let sighash = txn_cache
                            .legacy_signature_hash(
                                index,
                                &rune_sender.script_pubkey(),
                                EcdsaSighashType::All.to_u32(),
                            )
                            .unwrap();
                        let signature =
                            ecdsa_sign(sighash.as_byte_array().to_vec(), rune_sender_path.clone())
                                .await
                                .signature;
                        let mut signature = sec1_to_der(signature);
                        signature.push(EcdsaSighashType::All.to_u32() as u8);
                        let signature = PushBytesBuf::try_from(signature).unwrap();
                        let pubkey = PushBytesBuf::try_from(rune_sender_pubkey.clone()).unwrap();

                        input.script_sig = Builder::new()
                            .push_slice(signature)
                            .push_slice(pubkey)
                            .into_script();
                        input.witness.clear();
                    } else if index_of_utxos_to_be_signed_by_btc_sender.contains(&index) {
                        let sighash = txn_cache
                            .legacy_signature_hash(
                                index,
                                &bitcoin_sender.script_pubkey(),
                                EcdsaSighashType::All.to_u32(),
                            )
                            .unwrap();
                        let signature = ecdsa_sign(
                            sighash.as_byte_array().to_vec(),
                            bitcoin_sender_path.clone(),
                        )
                        .await
                        .signature;
                        let mut signature = sec1_to_der(signature);
                        signature.push(EcdsaSighashType::All.to_u32() as u8);
                        let signature = PushBytesBuf::try_from(signature).unwrap();
                        let pubkey = PushBytesBuf::try_from(bitcoin_sender_pubkey.clone()).unwrap();

                        input.script_sig = Builder::new()
                            .push_slice(signature)
                            .push_slice(pubkey)
                            .into_script();
                        input.witness.clear();
                    } else {
                        let sighash = txn_cache
                            .legacy_signature_hash(
                                index,
                                &fee_payer.script_pubkey(),
                                EcdsaSighashType::All.to_u32(),
                            )
                            .unwrap();
                        let signature =
                            ecdsa_sign(sighash.as_byte_array().to_vec(), fee_payer_path.clone())
                                .await
                                .signature;
                        let mut signature = sec1_to_der(signature);
                        signature.push(EcdsaSighashType::All.to_u32() as u8);
                        let signature = PushBytesBuf::try_from(signature).unwrap();
                        let pubkey = PushBytesBuf::try_from(fee_payer_pubkey.clone()).unwrap();

                        input.script_sig = Builder::new()
                            .push_slice(signature)
                            .push_slice(pubkey)
                            .into_script();
                        input.witness.clear();
                    }
                }

                let txid = txn.compute_txid().to_string();
                let txn_bytes = bitcoin::consensus::serialize(&txn);
                ic_cdk::println!("{}", hex::encode(&txn_bytes));

                bitcoin_send_transaction(SendTransactionRequest {
                    transaction: txn_bytes,
                    network,
                })
                .await
                .unwrap();

                SubmittedTxidType::Bitcoin { txid }
            }
        }
    }
}

async fn submit_txn(id: u128) {
    let txn = write_scheduled_state(|state| state.remove_txn(id));
    ic_cdk::println!("commit tx address: {}", txn.commit_tx_address);
    let network = read_config(|config| config.bitcoin_network());
    let utxos_response = bitcoin_get_utxos(GetUtxosRequest {
        network,
        address: txn.commit_tx_address.clone(),
        filter: None,
    })
    .await
    .unwrap()
    .0;
    let utxos = utxos_response.utxos;
    for utxo in utxos.iter() {
        ic_cdk::println!("bitcoin in utxo: {}", utxo.value);
    }
    if utxos.is_empty() {
        write_scheduled_state(|state| state.record_txn(id, txn));
        ic_cdk::trap("No UTXOs Found")
    }
    if utxos_response.tip_height - utxos[0].height < Runestone::COMMIT_CONFIRMATIONS as u32 {
        write_scheduled_state(|state| state.record_txn(id, txn));
        ic_cdk::trap("Not enough commit confirmation")
    }
    let transaction = bitcoin::consensus::serialize(&txn.txn);
    ic_cdk::println!("reveal: {}", hex::encode(&transaction));
    if bitcoin_send_transaction(SendTransactionRequest {
        network,
        transaction,
    })
    .await
    .is_err()
    {
        ic_cdk::println!("Timer was hit for reveal txn submission but failed to submit due to err");
        write_scheduled_state(|state| state.record_txn(id, txn));
    } else {
        ic_cdk::println!("transaction was submitted");
        ic_cdk_timers::clear_timer(txn.timer_id.into());
    }
}
