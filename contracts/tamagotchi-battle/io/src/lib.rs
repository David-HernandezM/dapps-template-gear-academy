#![no_std]

use gmeta::{Metadata, InOut, Out};
use gstd::{
    prelude::*, 
    msg, 
    ActorId, 
    collections::{BTreeSet, BTreeMap}, 
    exec,
    ReservationId
};
use tamagotchi_auto_io::{TmgAction, TmgEvent};
use tamagotchi_store_io::{StoreEvent, StoreAction, AttributeId};

pub type TamagotchiId = ActorId;
pub type Power = u16;
pub type Protection = u16;

const MAX_POWER: u16 = 10_000;
const MIN_POWER: u16 = 3_000;
const MAX_ENERGY: u16 = 60_000;
const MIN_ENERGY: u16 = 20_000;
const GAS_AMOUNT: u64 = 1_000_000_000; // 0.1 TVara
const TIME_FOR_UPDATE: u32 = 10; // 30 seconds 
// six steps so that each tamagotchi 
// can use its three attributes
const MAX_STEPS_FOR_ROUND: u8 = 4;

// Weapons
pub const SWORD_POWER: Power = 2;
pub const SWORD_ID: AttributeId = 1;

pub const KATANA_POWER: Power = 5;
pub const KATANA_ID: AttributeId = 2;

pub const AXE_POWER: Power = 4;
pub const AXE_ID: AttributeId = 3;

pub const MALLET_POWER: Power = 3;
pub const MALLET_ID: AttributeId = 4;

pub const FORK_POWER: Power = 1;
pub const FORK_ID: AttributeId = 5;

pub const BOMB_POWER: Power = 5;
pub const BOMB_ID: AttributeId = 6;

pub const KARATE_POWER: Power = 3;
pub const KARATE_ID: AttributeId = 7;

pub const BOX_POWER: Power = 2;
pub const BOX_ID: AttributeId = 8;

pub const KNIFE_POWER: Power = 1;
pub const KNIFE_ID: AttributeId = 9;

pub const SPOON_POWER: Power = 1;
pub const SPOON_ID: AttributeId = 10;

// Shields

pub const SHIELD_PROTECTION: Protection = 3_500;
pub const SHIELD_ID: AttributeId = 11;

pub const JACKET_PROTECTION: Protection = 2_000;
pub const JACKET_ID: AttributeId = 12;

pub const HELMET_PROTECTION: Protection = 1_500;
pub const HELMET_ID: AttributeId = 13;


pub struct ProgramMetadata;

impl Metadata for ProgramMetadata {
    type Init = InOut<BattleInit, String>;
    type Handle = InOut<BattleAction, BattleEvent>;
    type State = Out<Battle>;
    type Reply = ();
    type Others = InOut<BattleAction, BattleEvent>;
    type Signal = ();
}

#[derive(Encode, Decode, TypeInfo, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct BattleInit {
    pub tmg_store_id: ActorId
}

#[derive(Encode, Decode, TypeInfo, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Battle {
    pub players: Vec<Player>,
    pub state: BattleState,
    pub current_turn: u8,
    pub tmg_store_id: ActorId,
    pub winner: ActorId,
    pub steps: u8,
    pub weapons_data: BTreeMap<AttributeId, Power>,
    pub shields_data: BTreeMap<AttributeId, Protection>,
    pub reservations: Vec<ReservationId>,
}

impl Battle {
    pub async fn register(&mut self, tmg_id: &TamagotchiId, attributes: AttributesPerRound) {
        assert_eq!(
            self.state,
            BattleState::Registration,
            "The game has already started"
        );
    
        let owner = Self::get_owner(tmg_id).await;
        //let attributes = Self::get_attributes(&self.tmg_store_id, tmg_id).await;
        
        let power = Self::genetate_random_num_field(MIN_POWER, MAX_POWER);
        let power = MAX_POWER - power;
        let energy = Self::genetate_random_num_field(MIN_ENERGY, MAX_ENERGY);
        let energy = MAX_ENERGY - energy;
        let random_position = Self::get_turn();
        let actual_side = if random_position == 0 {
            DirectionOfMovement::Left
        } else {
            DirectionOfMovement::Right
        };
        
        let player = Player {
            owner,
            tmg_id: *tmg_id,
            energy,
            power,
            attributes,
            actual_attribute: 1,
            actual_side
        };
        
        self.players.push(player);
        
        if self.players.len() == 2 {
            self.current_turn = Self::get_turn();
            self.state = BattleState::Moves;
        }
        
        msg::reply(BattleEvent::Registered { tmg_id: *tmg_id }, 0)
            .expect("Error during a reply `BattleEvent::Registered");
    }
    
    pub async fn get_owner(tmg_id: &ActorId) -> ActorId {
        let reply: TmgEvent = msg::send_for_reply_as(*tmg_id, TmgAction::TmgInfo, 0, 0)
            .expect("Error in sending a message `TmgAction::TmgInfo")
            .await
            .expect("Unable to decode TmgEvent");
        if let TmgEvent::Owner(owner) = reply {
            owner
        } else {
            panic!("Wrong received message");
        }
    }
    
    async fn get_attributes(
        tmg_store_id: &ActorId,
        tmg_id: &TamagotchiId,
    ) -> BTreeSet<AttributeId> {
        let reply: StoreEvent = msg::send_for_reply_as(
            *tmg_store_id,
            StoreAction::GetAttributes {
                tamagotchi_id: *tmg_id,
            },
            0,
            0,
        )
        .expect("Error in sending a message `StoreAction::GetAttributes")
        .await
        .expect("Unable to decode `StoreEvent`");
        if let StoreEvent::Attributes { attributes } = reply {
            attributes
        } else {
            panic!("Wrong received message");
        }
    }
    
    pub fn get_turn() -> u8 {
        let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
        let (random, _) = exec::random(random_input)
            .expect("Error in getting random number");
        random[0] % 2
    }
    
    // genetate_power was changed to "random_number", to use it on power and energy
    pub fn genetate_random_num_field(min_field: u16, max_field: u16) -> u16 {
        let random_input: [u8; 32] = array::from_fn(|i| i as u8 + 1);
        let (random, _) = exec::random(random_input)
            .expect("Error in getting random number");
        let bytes: [u8; 2] = [random[0], random[1]];
        let random_number: u16 = u16::from_be_bytes(bytes) % max_field;
        if random_number < min_field {
            return max_field / 2;   
        }
        random_number
    }
    
    pub fn make_move(&mut self, direction: DirectionOfMovement) {
        assert_eq!(
            self.state,
            BattleState::Moves,
            "The game is not in `Moves` state"
        );
        let turn = self.current_turn as usize;
    
        let next_turn = (( turn + 1 ) % 2) as usize;
        
        let mut player = self.players[turn].clone();
        
        assert_eq!(
            player.owner,
            msg::source(),
            "You are not in the game or it is not your turn"
        );
        
        let mut opponent = self.players[next_turn].clone();
        
        let player_attribute_id = Self::player_actual_attribute_id(&player);
        
        player.actual_attribute += 1;
        player.actual_side = direction;
        
        // If on the player's turn he has a shield, he will only update 
        // which side he is on, since he will only defend himself for the next turn.
        if !self.shields_data.contains_key(&player_attribute_id) {
            let total_atack = self.tamagotchi_total_atack(&player, &opponent);
            
            // If both tamagotchi are on the same side, the opponent can be attacked, 
            // if not, the tamagotchi will not receive the attack.
            if player.actual_side == opponent.actual_side {
                opponent.energy = opponent.energy.saturating_sub(total_atack);
            } else {
                msg::send(player.owner, BattleEvent::OpponentDodgedTheAttack, 0)
                    .expect("error sending message to player");
            }
        }
    
        self.players[next_turn] = opponent.clone();
        self.players[turn] = player.clone();
        
        // Check if the opponent lost
        if opponent.energy == 0 {
            self.players = Vec::new();
            self.state = BattleState::GameIsOver;
            self.winner = player.tmg_id;
            self.steps = 0;
            
            msg::send(opponent.owner, BattleEvent::GameIsOver, 0)
                .expect("Error sending message to opponent");
            
            msg::reply(BattleEvent::GameIsOver, 0)
                .expect("Error in sending a reply `BattleEvent::GameIsOver`");
            return;
        }
        
        if self.steps <= MAX_STEPS_FOR_ROUND {
            self.steps += 1;
            self.current_turn = next_turn as u8;
            
            msg::reply(BattleEvent::MoveMade, 0)
                .expect("Error in sending a reply `BattleEvent::MoveMade`");
        } else {
            self.state = BattleState::Waiting;
            self.steps = 0;
            self.players[turn].actual_attribute = 1;
            self.players[next_turn].actual_attribute = 1;
            
            if !self.reservations.is_empty() {
                let Some(reservation_id) = self.reservations.pop() else {
                    panic!("Error getting transaction id, reservations is empty");  
                };
                
                msg::send_delayed_from_reservation(
                    reservation_id,
                    exec::program_id(),
                    BattleAction::UpdateInfo,
                    0,
                    TIME_FOR_UPDATE,
                )
                .expect("Error in sending a delayed message `BattleAction::UpdateInfo`");
            } else {
                msg::send_with_gas_delayed(
                    exec::program_id(),
                    BattleAction::UpdateInfo,
                    GAS_AMOUNT,
                    0,
                    TIME_FOR_UPDATE,
                )
                .expect("Error in sending a delayed message `BattleAction::UpdateInfo`");
            }
            
            msg::send(opponent.owner, BattleEvent::GoToWaitingState, 0)
                .expect("Error sending message to opponent");
            
            msg::reply(BattleEvent::GoToWaitingState, 0)
                .expect("Error in sending a reply `BattleEvent::MoveMade`");
        }
    }
    
    pub fn tamagotchi_total_atack(&self, player: &Player, opponent: &Player) -> u16 {
        let mut total_attack = player.power;
        let player_weapon_attribute = Self::player_actual_attribute_id(player);
        let opponent_attribute = Self::player_actual_attribute_id(opponent);
        
        // If the player has a weapon, his power is multiplied 
        // by the weapon at the final attack
        if self.weapons_data.contains_key(&player_weapon_attribute) {
            total_attack *= *self.weapons_data.get(&player_weapon_attribute).unwrap();
        }
        
        // If the opponent has a shield, protection 
        // points are subtracted from the final attack
        if self.shields_data.contains_key(&opponent_attribute) {
            let protection = *self.shields_data.get(&opponent_attribute).unwrap();
            total_attack = total_attack.saturating_sub(protection);
        }
        
        total_attack
    }
    
    pub fn player_actual_attribute_id(player: &Player) -> u32{
        match player.actual_attribute {
            1 => {
                if let Some(attribute_id) = player.attributes.round_1 {
                    attribute_id
                } else {
                    0
                }
            },
            2 => {
                if let Some(attribute_id) = player.attributes.round_2 {
                    attribute_id 
                } else {
                    0
                }
            },
            3 => {
                if let Some(attribute_id) = player.attributes.round_3 {
                    attribute_id
                } else {
                    0
                }
            },
            _ => 0
        }
    }
    
    pub fn update_tamagotchi_attributes(&mut self, new_attributes: AttributesPerRound) {
        assert_eq!(
            self.state,
            BattleState::Waiting,
            "The contract must be in `Waiting` state"
        );
        
        let caller = msg::source();
        
        if self.players[0].owner == caller {
            self.players[0].attributes = new_attributes;
        } else if self.players[1].owner == caller {
            self.players[1].attributes = new_attributes;
        } else {
            panic!("You are not in the game or it is not your turn");
        }
        
        msg::reply(BattleEvent::AttributesUpdated, 0)
            .expect("Error sending reply");
    }
    
    pub async fn update_info(&mut self) {
        assert_eq!(
            msg::source(),
            exec::program_id(),
            "Only the contract itself can call that action"
        );
        assert_eq!(
            self.state,
            BattleState::Waiting,
            "The contract must be in `Waiting` state"
        );
        
        self.state = BattleState::Moves;
        self.current_turn = Self::get_turn();
        
        msg::send(self.players[0].owner, BattleEvent::InfoUpdated, 0)
            .expect("Error during a reply BattleEvent::InfoUpdated");
            
        msg::send(self.players[1].owner, BattleEvent::InfoUpdated, 0)
            .expect("Error during a reply BattleEvent::InfoUpdated");
    }
    
    pub fn reset_contract(&mut self) {
        assert_eq!(
            self.state,
            BattleState::GameIsOver,
            "The contract must be in `GameIsOver` state"
        );
        
        self.state = BattleState::Registration;
        
        msg::reply(BattleEvent::ContractReinstated, 0)
            .expect("Error sending reply");
    }
    
    pub fn make_reservation(&mut self, reservation_amount: u64, reservation_duration: u32) {
        let reservation_id = ReservationId::reserve(
            reservation_amount,
            reservation_duration
        ).expect("reservation across executions");
        
        self.reservations.push(reservation_id);
        
        msg::reply(BattleEvent::GasReserved, 0)
            .expect("Error in reply");
    }
}

#[derive(Encode, Decode, TypeInfo, Default, Eq, PartialEq, Debug)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum BattleState {
    #[default]
    Registration,
    Moves,
    Waiting,
    GameIsOver,
}

#[derive(Encode, Decode, TypeInfo, Default, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Player {
   pub owner: ActorId,
   pub tmg_id: TamagotchiId,
   pub energy: u16,
   pub power: u16,
   pub attributes: AttributesPerRound,
   pub actual_attribute: u8,
   pub actual_side: DirectionOfMovement
}

#[derive(Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum BattleAction {
    Register {
        tamagotchi_id: TamagotchiId,
        attributes: AttributesPerRound
    },
    Move(DirectionOfMovement),
    UpdateInfo,
    StartNewGame,
    SendNewAttributesToNextRound {
        new_attributes: AttributesPerRound,
    },
    ReserveGas {
        reservation_amount: u64,
        duration: u32,
    },
}

#[derive(Encode, Decode, TypeInfo, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum BattleEvent {
    #[default]
    InfoUpdated,
    MoveMade,
    GoToWaitingState,
    GameIsOver,
    Registered {
        tmg_id: TamagotchiId
    },
    AttributesUpdated,
    ContractReinstated,
    GasReserved,
    OpponentDodgedTheAttack
}

// If the user does not have attributes in the store, 
// any number can be entered, or omitted
#[derive(Encode, Decode, TypeInfo, Default, Clone)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct AttributesPerRound {
    round_1: Option<AttributeId>,
    round_2: Option<AttributeId>,
    round_3: Option<AttributeId>,
}

#[derive(Encode, Decode, TypeInfo, Default, Clone, Eq, PartialEq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum DirectionOfMovement {
    #[default]
    Right,
    Left
}