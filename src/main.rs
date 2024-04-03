mod model;
mod p2p;

use crate::p2p::{AppBehaviour, EventType::Input, EventType::LocalChainResponse, KEYS, PEER_ID};
use model::{calculate_hash, hash_to_binary_representation, Block, DIFFICULTY_PREFIX};

use chrono::prelude::*;
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    Transport,
};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::mpsc,
    time::sleep,
};

pub struct App {
    pub blocks: Vec<Block>,
}

impl App {
    pub fn new() -> Self {
        Self { blocks: vec![] }
    }

    pub fn genesis(&mut self) {
        // TODO: Should I move this genesis method to separated file?
        let genesis_block = Block {
            id: 0,
            timestamp: Utc::now().timestamp(),
            curr_hash: String::from(
                "433855b7d2b96c23a6f60e70c655eb4305e8806b682a9596a200642f947259b1",
            ),
            prev_hash: String::from("0"),
            // signature: 123,
            data: String::from("0"),
            nonce: 1234,
        };

        self.blocks.push(genesis_block)
    }

    pub fn is_block_valid(&self, block: &Block, prev_block: &Block) -> bool {
        if block.prev_hash != prev_block.curr_hash {
            warn!("Block with id: {} has wrong previous hash", block.id);
            return false;
        } else if !hash_to_binary_representation(
            &hex::decode(&block.curr_hash).expect("Can decode from hex"),
        )
        .starts_with(DIFFICULTY_PREFIX)
        {
            warn!("Block with id: {} has invalid difficulty", block.id);
            return false;
        } else if block.id != prev_block.id + 1 {
            warn!(
                "Block with id: {}, is not the next block after the latest: {}",
                block.id, prev_block.id
            );
            return false;
        } else if hex::encode(calculate_hash(
            block.id,
            block.timestamp,
            &block.prev_hash,
            // &block.curr_hash,
            &block.data,
            block.nonce,
            // block.signature,
        )) != block.curr_hash
        {
            // TODO: Here can be an error. Check it out
            warn!("Block with id: {} has invalid hash", block.id);
            return false;
        }

        true
    }

    pub fn is_chain_valid(&self, chain: &[Block]) -> bool {
        for i in 0..chain.len() {
            if i == 0 {
                continue;
            }

            let first = chain.get(i - 1).expect("has to exist");
            let second = chain.get(i).expect("has to exist");

            if !self.is_block_valid(second, first) {
                return false;
            }
        }

        true
    }

    pub fn add_block(&mut self, block: Block) {
        let latest_block = self.blocks.last().expect("There is at least one block");

        if self.is_block_valid(&block, latest_block) {
            self.blocks.push(block);
        } else {
            error!("Cannot add block: invalid");
        }
    }

    pub fn choose_chain(&mut self, local: Vec<Block>, remote: Vec<Block>) -> Vec<Block> {
        let is_local_valid = self.is_chain_valid(&local);
        let is_remote_valid = self.is_chain_valid(&remote);

        if is_local_valid && is_remote_valid {
            if local.len() >= remote.len() {
                local
            } else {
                remote
            }
        } else if is_remote_valid && !is_local_valid {
            remote
        } else if !is_remote_valid && is_local_valid {
            local
        } else {
            panic!("Local and remote chains are both invalid!")
        }
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    print!("{}[2J", 27 as char);
    println!("{}", PEER_ID.clone());
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&KEYS)
        .expect("can create auth keys");

    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour = AppBehaviour::new(App::new(), response_sender, init_sender.clone()).await;

    let mut swarm = SwarmBuilder::new(transp, behaviour, *PEER_ID)
        .executor(Box::new(|fut| {
            spawn(fut);
        }))
        .build();

    let mut stdin = BufReader::new(stdin()).lines();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    spawn(async move {
        sleep(Duration::from_secs(1)).await;
        println!("sending init event");
        init_sender.send(true).expect("can send init event");
    });

    loop {
        let evt = {
            select! {
                line = stdin.next_line() => Some(p2p::EventType::Input(line.expect("can get line").expect("can read line from stdin"))),
                response = response_rcv.recv() => {
                    Some(LocalChainResponse(response.expect("response exists")))
                },
                _init = init_rcv.recv() => {
                    Some(p2p::EventType::Init)
                }
                event = swarm.select_next_some() => {
                    // println!("Unhandled Swarm Event: {:#?}", event);
                    None
                },
            }
        };

        if let Some(event) = evt {
            match event {
                p2p::EventType::Init => {
                    let peers = p2p::get_list_peers(&swarm);
                    swarm.behaviour_mut().app.genesis();

                    println!("Connected nodes: {}", peers.len());
                    if !peers.is_empty() {
                        let req = p2p::LocalChainRequest {
                            from_peer_id: peers
                                .iter()
                                .last()
                                .expect("at least one peer")
                                .to_string(),
                        };

                        let json = serde_json::to_string(&req).expect("can jsonify request");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                    }
                }
                p2p::EventType::LocalChainResponse(resp) => {
                    let json = serde_json::to_string(&resp).expect("can jsonify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(p2p::CHAIN_TOPIC.clone(), json.as_bytes());
                }
                p2p::EventType::Input(line) => match line.as_str() {
                    cmd if cmd.starts_with("/help") => p2p::help_message(),
                    cmd if cmd.starts_with("/list peers") => p2p::handle_print_peers(&swarm),
                    cmd if cmd.starts_with("/list chain") => p2p::handle_print_chain(&swarm),
                    cmd if cmd.starts_with("create b") => p2p::handle_create_block(cmd, &mut swarm),
                    cmd if cmd.starts_with("/clear") => p2p::clear_chat(),
                    _ => error!("unknown command"),
                },
            }
        }
    }
}
