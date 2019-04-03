use actix::prelude::*;
use std::collections::{HashMap};
use serde_json::{Result as JSON_Result};
use serde::{Deserialize, Serialize};
use redis::Commands;
use set::Set;

/// Server sends this messages to session
#[derive(Message)]
pub struct Message(pub String);

#[derive(Serialize, Deserialize)]
pub struct ClientUser {
    name: String,
    points: isize,
}

#[derive(Serialize, Deserialize)]
pub struct UserMessage {
    eventType: String,
    users: Vec<ClientUser>
}

#[derive(Serialize, Deserialize)]
pub struct GameTypeMessage {
    eventType: String,
    gameType: String,
}

#[derive(Serialize, Deserialize)]
pub struct GameUpdateMessage {
    eventType: String,
    gameState: Game,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    name: String,
    points: isize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Selection {
    user: String,
    valid: bool,
    selection: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Game {
    numberOfSets: usize,
    deck: String,
    board: String,
    previousSelection: Option<Selection>
}

#[derive(Serialize, Deserialize)]
pub struct Lobby {
    users: HashMap<usize, User>,
    game_type: Option<String>,
    game_state: Option<Game>,
}

/// `Server` manages rooms and responsible for coordinating game
/// sessions. implementation is super primitive
pub struct Server {
    sessions: HashMap<String, Lobby>,
    // TODO: rename user_addrs to sessions
    user_addrs: HashMap<usize, Recipient<Message>>,
    redis: redis::Client
}

impl Default for Server {
    fn default() -> Server {
        Server {
            sessions: HashMap::new(),
            user_addrs: HashMap::new(),
            redis:  redis::Client::open("redis://redis:6379").unwrap(),
        }
    }
}

/// Make actor from `Server`
impl Actor for Server {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

#[allow(unused_must_use)]
impl Server {
    fn emit_users(&mut self, users: HashMap<usize, User>) {
        let mut message = UserMessage {
            eventType: "users".to_string(),
            users: Vec::new(),
        };

        for user in users.values() {
            message.users.push(
                ClientUser {
                    name: user.name.clone(),
                    points: user.points.clone(),
                }
            )
        }

        let message_string = serde_json::to_string(&message).unwrap();
        for (id, _user) in users {
            // TODO continue if skip_id == user key
            match self.user_addrs.get(&id) {
                Some(addr) => addr.do_send(Message(message_string.to_owned())),
                None => Ok(()) // user is not connected to this server
            };
        }
    }

    fn emit_game_type(&mut self, game_type: &String, users: HashMap<usize, User>) {
        let message = GameTypeMessage {
            eventType: "setGameType".to_string(),
            gameType: game_type.to_string(),
        };
        let message_string = match serde_json::to_string(&message) {
            JSON_Result::Ok(u) => u,
            _ => panic!("Not able to serialize users")
        };
        for (id, _user) in users {
            // TODO continue if skip_id == user key
            match self.user_addrs.get(&id) {
                Some(addr) => addr.do_send(Message(message_string.to_owned())),
                None => Ok(()) // user is not connected to this server
            };
        }
    }

    fn emit_game_update(&mut self, room_name: &String) {
        let session = self.sessions.get(room_name).unwrap();
        let game_state = session.game_state.as_ref().unwrap();

        let message = GameUpdateMessage {
            eventType: "updateGame".to_string(),
            gameState: game_state.clone(),
        };
        let message_string = match serde_json::to_string(&message) {
            JSON_Result::Ok(u) => u,
            _ => panic!("Not able to serialize users")
        };
        for (id, _user) in &session.users {
            // TODO continue if skip_id == user key
            match self.user_addrs.get(&id) {
                Some(addr) => addr.do_send(Message(message_string.to_owned())),
                None => Ok(()) // user is not connected to this server
            };        }
    }
}

/// Join room, if room does not exists create new one.
#[derive(Message)]
pub struct Join {
    pub id: usize,
    pub addr: Recipient<Message>,
    pub username: String,
    pub room_name: String,
}

// Join room, send disconnect message to old room
/// send join message to new room
impl Handler<Join> for Server {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, addr, username, room_name } = msg;
  
        // add refrence to user addr to server session
        self.user_addrs.insert(id, addr);

        // get/create lobby
        let con = self.redis.get_connection().unwrap();
        let existing_lobby: redis::RedisResult<String> = con.get(&room_name);
        let mut new_lobby: Lobby = match existing_lobby {
            Ok(l) => {
                serde_json::from_str(&l).unwrap()
            },
            _ => Lobby {
                users: HashMap::new(),
                game_type: None,
                game_state: None
            }
        };

        // add new user to lobby
        new_lobby.users.insert(
            id,
            User {
                name: username,
                points: 0
            }
        );
        let lobby_json = serde_json::to_string(&new_lobby).unwrap();
        let _: () = con.set(&room_name, lobby_json).unwrap();

         // TODO: call emit functions from redis subscription
        self.emit_users(new_lobby.users);
    }
}


#[derive(Message)]
pub struct SetGameType {
    pub game_type: String,
    pub room_name: String,
}

// Set game type, and broadcast that type to all clients in the room
impl Handler<SetGameType> for Server {
    type Result = ();

    fn handle(&mut self, msg: SetGameType, _: &mut Context<Self>) {
        let SetGameType { game_type, room_name } = msg;

        // get lobby
        let con = self.redis.get_connection().unwrap();
        let existing_lobby: redis::RedisResult<String> = con.get("room_name");
        let mut new_lobby: Lobby = match existing_lobby {
            Ok(l) => {
                serde_json::from_str(&l).unwrap()
            },
            _ => panic!("requested lobby does not exist")
        };

        // update lobby game type
        new_lobby.game_type = Some(game_type.clone());
        let lobby_json = serde_json::to_string(&new_lobby).unwrap();
        let _: () = con.set(&room_name, lobby_json).unwrap();

         // TODO: call emit functions from redis subscription
        self.emit_game_type(&game_type, new_lobby.users);
    }
}


#[derive(Message)]
pub struct StartGame {
    pub room_name: String,
}

// Set game type, and broadcast that type to all clients in the room
impl Handler<StartGame> for Server {
    type Result = ();

    fn handle(&mut self, msg: StartGame, _: &mut Context<Self>) {
        let StartGame { room_name } = msg;

        if self.sessions.get_mut(&room_name).is_none() {
            return
        }

        let set = Set::new(4, 3);
        let deck = set.init_deck();
        let update_board = set.update_board(deck, "".to_string());
        let game_state = Game {
            deck: update_board.get_deck(),
            board: update_board.get_board(),
            numberOfSets: update_board.sets,
            previousSelection: None
        };
        self.sessions.get_mut(&room_name).unwrap().game_state = Some(game_state);

        self.emit_game_update(&room_name);
    }
}


#[derive(Message)]
pub struct VerifySet {
    pub id: usize,
    pub room_name: String,
    pub selected: String,
}

// Set game type, and broadcast that type to all clients in the room
impl Handler<VerifySet> for Server {
    type Result = ();

    fn handle(&mut self, msg: VerifySet, _: &mut Context<Self>) {
        let VerifySet { id, room_name, selected } = msg;
        
        if self.sessions.get_mut(&room_name).is_none() {
            return
        }
        
        let set = Set::new(4, 3);
        let is_valid_set = set.is_set(selected.clone());
        if is_valid_set {
            {
                let user = self.sessions.get_mut(&room_name).unwrap()
                            .users.get_mut(&id).unwrap();
                user.points = user.points + 1;
            }

            // self.emit_users(&room_name, id, self.sessions.get(&room_name).unwrap().users);

            let name = self.sessions.get(&room_name).unwrap().users.get(&id).unwrap().name.to_string();

            let old_board = self.sessions.get(&room_name).unwrap().game_state.as_ref().unwrap().board.to_string();
            let deck  = self.sessions.get(&room_name).unwrap().game_state.as_ref().unwrap().deck.to_string();

            let mut board: Vec<&str> = old_board.split(",").collect();
            for card in selected.split(",") {
                board.remove_item(&card);
            }

            let set = Set::new(4, 3);
            let update_board = set.update_board(deck, board.join(","));
            let game_state = Game {
                deck: update_board.get_deck(),
                board: update_board.get_board(),
                numberOfSets: update_board.sets,
                previousSelection: Some(Selection{
                    user: name,
                    valid: true,
                    selection: selected.to_string(),
                }),
            };

            self.sessions.get_mut(&room_name).unwrap().game_state = Some(game_state);

            self.emit_game_update(&room_name);
        } else {
            {
                let user = self.sessions.get_mut(&room_name).unwrap()
                            .users.get_mut(&id).unwrap();
                user.points = user.points - 1;
            }

            // self.emit_users(&room_name, id, self.sessions.get(&room_name).unwrap().users);

            let name = self.sessions.get(&room_name).unwrap().users.get(&id).unwrap().name.to_string();

            let previousSelection = Some(Selection{
                user: name,
                valid: false,
                selection: selected,
            });

            self.sessions.get_mut(&room_name).unwrap().game_state.as_mut().unwrap().previousSelection = previousSelection;

            self.emit_game_update(&room_name);
        }
    }
}