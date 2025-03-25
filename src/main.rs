use local_ip_address::local_ip;
use std::{ 
    net::{ TcpListener, TcpStream },
    io::{ Write, Read },
    fmt::Display,
    thread::scope,
    sync::mpsc::channel,
    fs::{ OpenOptions, File },
    collections::HashMap,
};
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
enum Cmd {
    SendResults,
    Query,
    Reset
}
struct Answers(Vec<HashMap<u16, (Option<Answer>, Option<Answer>)>>);

const PATH_ONE: &'static str = "./one";
const PATH_TWO: &'static str = "./two";

fn main() {
    let ip = local_ip().unwrap();
    let listener = TcpListener::bind((ip, 6942u16)).unwrap();
    let mut connections: Vec<TcpStream> = Vec::with_capacity(20);
    let answers = Answers::from_files();
    //let (tx, rx) = channel::<[u8; 2]>();
    /*scope(|s| {
       s.spawn(move || {*/
    let file_one = OpenOptions::new()
        .append(true)
        .create(true)
        .open(PATH_ONE);
    let file_one = OpenOptions::new()
        .append(true)
        .create(true)
        .open(PATH_TWO);
    for stream in listener.incoming() {
        //NOTE:
        //Byte convention:
        //  0: 0 if client, 1 if control
        //  1: 0 if read, 1 if write
        //  2: 0 if side 1, 1 if side 2
        //  3-4: answer
        //  5-15: id
        let mut buf: [u8; 2] = [0; 2];
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
                    let res1 = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(PATH_ONE);
                    let res2 = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open(PATH_TWO);
                    if let Err(err) = res1 {
                        println!("Failed truncating file 1: {}", err);
                    }
                    if let Err(err) = res2 {
                        println!("Failed truncating file 2: {}", err);
                    }
                },
                SendResults => {
                    let answers = Answers::from_files();
                },
                Query => {
                        
                },
            }
        }
        let is_write = match byte1 & 0b01000000 {
            0 => false,
            _ => true,
        };
        let side = match byte1 & 0b00100000 {
            0 => Side::One,
            _ => Side::Two,
        };
        let answer = match (byte1 & 0b00010000, byte1 & 0b00001000) {
            (0, 0) => Answer::A,
            (0, _) => Answer::B,
            (_, 0) => Answer::C,
            (_, _) => Answer::D
        };
        println!("Answer: {}", answer);
        let mut id = buf[1] as u16;
        id |= ((byte1 & 0b00000111) as u16) << 8;
        println!("ID: {}", id);
        println!("Side: {}", side);
        println!("Is write: {}", is_write);
        println!("Buf: {:?}", buf);
        connections.push(stream);
    }
        //}); 
        /*s.spawn(move || {
            for cmd in rx.iter() {
                let byte1 = cmd[0];
            }    
        });*/
    //});
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
            0 => Query,
            1 => Reset,
            _ => SendResults
        }
    }
}

impl Answers {
    fn new() -> Answers {
        Answers(Vec::new())
    }
    fn from_files() /*-> Answers*/ {
        let mut file1 = OpenOptions::new()
            .read(true)
            .open(PATH_ONE);
        let mut file2 = OpenOptions::new()
            .read(true)
            .open(PATH_TWO);
        let mut answers_one: HashMap<u16, Vec<Answer>> = HashMap::new();
        let mut answers_two: HashMap<u16, Vec<Answer>> = HashMap::new();
        let mut buf1 = [0u8; 4096];
        let mut buf2 = [0u8; 4096];
        if let Ok(mut file) = file1 {
            let _ = file.read(&mut buf1);        
        };
        if let Ok(mut file) = file2 {
            let _ = file.read(&mut buf2);        
        };
    }
    fn append(&mut self, id: u16, side: Side, answer: Answer) {

    }
    fn answer_counts(&self) -> Vec<[u8; 2]> {

    }
}
