use local_ip_address::local_ip;
use std::{ 
    net::{ TcpListener, TcpStream },
    io::{ Write, Read },
    fmt::{ Display, Debug, Formatter },
    thread::scope,
    sync::mpsc::channel,
    fs::{ OpenOptions, File },
    collections::HashMap,
};
#[derive(Debug, Clone, Copy)]
enum Answer {
    A,
    B,
    C,
    D
}
use Answer::*;
enum Side {
    One,
    Two
}
use Side::*;
enum Cmd {
    SendResults,
    Query,
    Reset,
    Listening
}
use Cmd::*;
struct Answers(Vec<HashMap<u8, [Option<Answer>; 2]>>);
impl Debug for Answers {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self.0)
    }
}

const PATH_ONE: &'static str = "./one";
const PATH_TWO: &'static str = "./two";

fn main() {
    let ip = local_ip().unwrap();
    let listener = TcpListener::bind((ip, 6942u16)).unwrap();
    let mut connections: HashMap<u8, [Option<TcpStream>; 2]> = HashMap::new();
    let mut answers = Answers::new();
    for stream in listener.incoming() {
        //NOTE:
        //Byte convention:
        //  0: 0 if client, 1 if control
        //  1: 0 if read, 1 if write
        //  2: 0 if side 1, 1 if side 2
        //  3-4: answer
        //  5-15: id
        let mut buf: [u8; 2] = [0; 2];
        if let Err(_) = stream {
            continue;
        }
        if let Err(_) = stream {
            continue;
        }
        let mut stream = stream.unwrap();
        let res = stream.read(&mut buf);
        if let Err(_) = res {
            continue;
        }
        let byte1 = buf[0];
        let id = buf[1];
        let is_control = match byte1 & 0b10000000 {
            0 => false,
            _ => true,
        };
        if is_control {
            match Cmd::new(byte1) {
                Reset => {
                    println!("Resetting answers");
                    answers.reset();
                    connections.clear();
                },
                SendResults => {
                    println!("Sending results to clients");
                    //Send results to every subscriber
                    let inverted = answers.invert();
                    for (id, conns) in &mut connections {
                        let mut bytes1 = [0, 0, 0];
                        let mut bytes2 = [0, 0, 0];
                        let answer_vec = inverted.get(&id).unwrap();
                        for (j, answer_pair) in answer_vec.iter().enumerate() {
                            let bits1 = match answer_pair[0] {
                                Some(A) => 0,
                                Some(B) => 1,
                                Some(C) => 2,
                                Some(D) => 3,
                                None => 0
                            } << (j * 2);
                            let bits2 = match answer_pair[1] {
                                Some(A) => 0,
                                Some(B) => 1,
                                Some(C) => 2,
                                Some(D) => 3,
                                None => 0
                            } << (j * 2);
                            bytes1[j/4] |= bits1;
                            bytes2[j/4] |= bits2;
                        }
                        if let Some(conn1) = &mut conns[0] {
                            let _ = conn1.write(&bytes2);
                        }
                        if let Some(conn2) = &mut conns[1] {
                            let _ = conn2.write(&bytes1);
                        }
                    }
                },
                Query => {
                    println!("Responding to query");
                    let flattened = answers.query().into_flattened();
                    let _ = stream.write(flattened.as_slice());
                },
                Listening => {
                    println!("Subscribing client");
                    let side = Side::from(byte1 & 0b00000100);
                    let user = connections.get(&id);
                    if let None = user {
                        let _ = connections.insert(id, [None, None]);
                    }
                    let user = connections.get_mut(&id).unwrap();
                    match side {
                        One => {user[0] = Some(stream);},
                        Two => {user[1] = Some(stream);}
                    };
                },
            }
            continue;
        }
        let side = Side::from(byte1 & 0b01000000);
        let question_id = (byte1 & 0b00111100) >> 2;
        let answer = match (byte1 & 0b00000010, byte1 & 0b00000001) {
            (0, 0) => Answer::A,
            (0, _) => Answer::B,
            (_, 0) => Answer::C,
            (_, _) => Answer::D
        };
        answers.append(side, question_id, answer, id);
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let display = match self {
            Self::One => "One",
            Self::Two => "Two"
        };
        write!(f, "{}", display) 
    }
}
impl From<bool> for Side {
    fn from(val: bool) -> Self {
        use Side::*;
        match val {
            false => One,
            true => Two
        }
    }
}
impl From<u8> for Side {
    fn from(val: u8) -> Self {
        use Side::*;
        match val {
            0 => One,
            _ => Two,
        }
    }
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let display = match self {
            Answer::A => 'A',
            Answer::B => 'B',
            Answer::C => 'C',
            Answer::D => 'D',
        };
        write!(f, "{}", display) 
    }
}
impl From<Answer> for u8 {
    fn from(val: Answer) -> u8 {
        use Answer::*;
        match val {
            A => 0,
            B => 1,
            C => 2,
            D => 3
        }
    }
}

impl Cmd {
    fn new(byte1: u8) -> Self {
        use Cmd::*;
        let cmd_num = byte1 & 0b00000011;
        match cmd_num {
            0 => Listening,
            1 => Query,
            2 => Reset,
            _ => SendResults,
        }
    }
}

impl Answers {
    fn new() -> Answers {
        Answers(Vec::new())
    }
    fn append(&mut self, side: Side, question_id: u8, answer: Answer, id: u8) {
        let users_opt = self.0.get(question_id as usize);
        if let None = users_opt {
            self.0.insert(question_id as usize, HashMap::new());
        } 
        let users = self.0.get_mut(question_id as usize).unwrap();
        let answers_opt = users.get(&id);
        if let None = answers_opt {
            let _ = users.insert(id, [None, None]);
        }
        let answer_pair = users.get_mut(&id).unwrap();
        match side {
            One => {answer_pair[0] = Some(answer)},
            Two => {answer_pair[1] = Some(answer)},
        };
    }
    fn into_inner(self) -> Vec<HashMap<u8, [Option<Answer>; 2]>> {
        self.0
    }
    fn inner_mut(&mut self) -> &mut Vec<HashMap<u8, [Option<Answer>; 2]>> {
        &mut self.0
    }
    fn inner(&self) -> &Vec<HashMap<u8, [Option<Answer>; 2]>> {
        &self.0
    }
    fn reset(&mut self) {
        self.0.clear();
    }
    fn query(&self) -> Vec<[u8; 2]> {
        let mut vec: Vec<[u8; 2]> = Vec::new();
        for question in &self.0 {
            let mut count1 = 0u8;
            let mut count2 = 0u8;
            for answers in question {
                if let Some(_) = answers.1[0] { count1 += 1; }
                if let Some(_) = answers.1[1] { count2 += 1; }
            }
            let res = vec.push([count1, count2]);
        }
        vec
    }
    //Vec<HashMap<u8, [Option<Answer>; 2]>>
    fn invert(&self) -> HashMap<u8, Vec<[Option<Answer>; 2]>> {
        let mut hashmap: HashMap<u8, Vec<[Option<Answer>; 2]>> = HashMap::new();
        for question in &self.0 {
            for user in question.into_iter() {
                if !hashmap.contains_key(user.0) {
                    hashmap.insert(*user.0, Vec::new());
                }
                let answers = hashmap.get_mut(user.0).unwrap();
                //TODO:
                answers.push(*user.1);
            } 
        }
        hashmap
    }
}

//TODO:
//Make the control client actually do something
//Send results to clients
