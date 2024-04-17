use crate::internals::{ArgumentTypes, HashParams, VerifyParams};
use log::{error, info};
use samp::amx::AmxIdent;
use samp::plugin::SampPlugin;
use samp::prelude::*;
use std::collections::LinkedList;
use std::sync::mpsc::{channel, Receiver, Sender};
use threadpool::ThreadPool;

pub struct SampBcrypt {
    pub hashes: LinkedList<String>,
    pub pool: ThreadPool,
    pub hash_sender: Option<Sender<HashParams>>,
    pub hash_receiver: Option<Receiver<HashParams>>,
    pub verify_sender: Option<Sender<VerifyParams>>,
    pub verify_receiver: Option<Receiver<VerifyParams>>,
    pub amx_list: Vec<AmxIdent>,
}

impl SampPlugin for SampBcrypt {
    fn on_load(&mut self) {
        info!("Version: 0.4.0");
        let (verify_sender, verify_receiver) = channel();
        self.verify_sender = Some(verify_sender);
        self.verify_receiver = Some(verify_receiver);

        let (hash_sender, hash_receiver) = channel();
        self.hash_sender = Some(hash_sender);
        self.hash_receiver = Some(hash_receiver);
    }

    fn on_amx_load(&mut self, amx: &Amx) {
        self.amx_list.push(amx.ident());
    }

    fn on_amx_unload(&mut self, amx: &Amx) {
        let raw = amx.ident();
        let index = self.amx_list.iter().position(|x| *x == raw).unwrap();
        self.amx_list.remove(index);
    }

    fn process_tick(&mut self) {
        for (playerid, callback, hashed, optional_args) in
            self.hash_receiver.as_ref().unwrap().try_iter()
        {
            let mut executed = false;
            self.hashes.push_front(hashed);

            for amx in &self.amx_list {
                if let Some(amx) = samp::amx::get(*amx) {
                    let allocator = amx.allocator();

                    for param in optional_args.iter().rev() {
                        match param {
                            ArgumentTypes::Primitive(x) => {
                                if amx.push(x).is_err() {
                                    error!("*Cannot execute callback {:?}", callback);
                                }
                            }
                            ArgumentTypes::String(data) => {
                                let buf = allocator.allot_buffer(data.len() + 1).unwrap();
                                let amx_str = unsafe { AmxString::new(buf, data) };
                                if amx.push(amx_str).is_err() {
                                    error!("*Cannot execute callback {:?}", callback);
                                }
                            }
                        }
                    }
                    if amx.push(playerid).is_err() {
                        error!("*Cannot execute callback {:?}", callback);
                    }
                    if let Ok(index) = amx.find_public(&callback) {
                        if amx.exec(index).is_ok() {
                            executed = true;
                            break;
                        }
                    }
                }
            }
            if !executed {
                error!("*Cannot execute callback {:?}", callback);
            }
        }

        for (playerid, callback, success, optional_args) in
            self.verify_receiver.as_ref().unwrap().try_iter()
        {
            let mut executed = false;
            for amx in &self.amx_list {
                if let Some(amx) = samp::amx::get(*amx) {
                    let allocator = amx.allocator();

                    for param in optional_args.iter().rev() {
                        match param {
                            ArgumentTypes::Primitive(x) => {
                                if amx.push(x).is_err() {
                                    error!("*Cannot execute callback {:?}", callback);
                                }
                            }
                            ArgumentTypes::String(data) => {
                                let buf = allocator.allot_buffer(data.len() + 1).unwrap();
                                let amx_str = unsafe { AmxString::new(buf, data) };
                                if amx.push(amx_str).is_err() {
                                    error!("*Cannot execute callback {:?}", callback);
                                }
                            }
                        }
                    }
                    if amx.push(success).is_err() {
                        error!("*Cannot execute callback {:?}", callback);
                    }
                    if amx.push(playerid).is_err() {
                        error!("*Cannot execute callback {:?}", callback);
                    }
                    if let Ok(index) = amx.find_public(&callback) {
                        if amx.exec(index).is_ok() {
                            executed = true;
                            break;
                        }
                    }
                }
            }
            if !executed {
                error!("*Cannot execute callback {:?}", callback);
            }
        }

        self.hashes.clear();
    }
}
