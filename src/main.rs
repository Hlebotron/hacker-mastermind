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
#[derive(Debug)]
enum Answer {
    A,
    B,
    C,
    D
}
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
struct Answers(Vec<HashMap<u8, (Option<Answer>, Option<Answer>)>>);
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
    let mut connections1: Vec<TcpStream> = Vec::with_capacity(20);
    let mut connections2: Vec<TcpStream> = Vec::with_capacity(20);
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
        let mut stream = stream.unwrap();
        let _ = stream.read(&mut buf).unwrap();
        let byte1 = buf[0];
        let is_control = match byte1 & 0b10000000 {
            0 => false,
            _ => true,
        };
        if is_control {
            use Cmd::*;
            match Cmd::new(byte1) {
                Reset => {
                    println!("Resetting answers");
                    answers.reset();
                    connections1.clear();
                    connections2.clear();
                },
                SendResults => {
                    println!("Sending results to clients");
                    println!("{:?}", answers);
                    //Send results to every subscriber
                },
                Query => {
                    println!("Responding to query");

                },
                Listening => {
                    println!("Subscribing client");
                    let side = Side::from(byte1 & 0b00000001);
                    match side {
                        One => connections1.push(stream),
                        Two => connections2.push(stream)
                    }
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
        println!("Answer: {}", answer);
        let id = buf[1];
        println!("ID: {}", id);
        println!("Side: {}", side);
        println!("Buf: {:?}", buf);
        answers.append(side, question_id, answer, id);
    }
}
//TODO: Way to control remotely (And actually respond to the control commands)
//Commands:
//  Delete file contents
//  Stop counting people in
//TODO: In-memory track of people and questions
//
//
//NOTE: Project idea: Sender of arbitrary byte data via TcpStream

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

impl Cmd {
    fn new(byte1: u8) -> Self {
        use Cmd::*;
        let cmd_num = byte1 & 0b01111111;
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
            let _ = users.insert(id, (None, None));
        }
        let answer_pair = users.get_mut(&id).unwrap();
        match side {
            One => {answer_pair.0 = Some(answer)},
            Two => {answer_pair.1 = Some(answer)},
        };
        println!("Answers: {:?}", self);
    }
    fn into_inner(self) -> Vec<HashMap<u8, (Option<Answer>, Option<Answer>)>> {
        self.0
    }
    fn inner(&mut self) -> &mut Vec<HashMap<u8, (Option<Answer>, Option<Answer>)>> {
        &mut self.0
    }
    fn reset(&mut self) {
        self.0.clear();
    }
    fn answer_counts(&self) -> Vec<u8> {
        for question in &self.0 {
            
        }
        Vec::new()
    }
}

//TODO:
//Control client
//Error gives whether client has hung up or not (TcpStream)
