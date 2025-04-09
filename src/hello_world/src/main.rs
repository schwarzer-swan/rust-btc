use std::env;
use std::process::exit;
use std::thread::spawn;
use serde::Serialize;
use std::marker::PhantomData;
use std::thread;

fn print_mystring(s: &String) {
    println!("{}", s)
}

fn foobar<'a, 'b>(_x: &'a i32, _y: &'b i32)
where
    'a: 'b,
{
    println!("im in foobar")
}

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.bytes().len() > y.bytes().len() {
        x
    } else {
        y
    }
}

// compiler can not rule out cases where
// the name member is dropped befor the enclosing structure
#[derive(Debug)]
struct Person<'a> {
    name: &'a str,
}

// slice is a view of a collection regardless
// String literals are slices
fn my_reff() -> &'static str {
    "this is a static ref"
}

fn print_static<T: 'static>(value: T)
where
    T: std::fmt::Debug,
{
    println!("Value: {:?}", value)
}
struct Hello;

trait SayHi {
    fn say_hi(self);
}
impl SayHi for Hello {
    fn say_hi(self) {
        println!("This hi will cost my life - I am owned value");
    }
}
// references are distict types
impl SayHi for &Hello {
    fn say_hi(self) {
        println!("Hi, I am a reference to Hello");
    }
}

impl SayHi for &&Hello {
    fn say_hi(self) {
        println!("Hi, I am a double reference to Hello");
    }
}
struct MyStruct<'a> {
    remainder: Option<&'a str>,
}
impl<'a> MyStruct<'a> {
    fn pop_first_char_as_string(&mut self) -> Option<&str> {
        let remainder: &mut &str = &mut self.remainder?;
        let c: &str = &remainder[0..1];
        if remainder.len() != 1 {
            *remainder = &remainder[1..];
            Some(c)
        } else {
            self.remainder.take()
        }
        // self.remainder.take()
    }
}
trait Quack {
    fn quack(&self);
}
struct Duck;

struct FormalDuck {
    name: String,
}
impl FormalDuck {
    // create a new duck
    fn new(name: String) -> Self {
        Self { name }
    }
}
impl Quack for FormalDuck {
    fn quack(&self) {
        println!(
            "Good evening, ladies and gentlemen, my name is {}. \
Without further ado: quack",
            self.name
        );
    }
}
impl Quack for Duck {
    fn quack(&self) {
        println!("quack");
    }
}

// static dispatch with trait constraints
fn ducks_say<T>(quacker: T)
where
    T: Quack,
{
    quacker.quack()
}

// dynamic dispatch
fn ducks_say_dyn(dock: &dyn Quack) {
    dock.quack()
}

fn no_param<T>(_:T) {}

fn my_function2(i: &mut  i32) {
    *i *= 2;
    println!("my_function({:?})", i)
}
fn my_function(mut i:  i32) {
    i *= 2;
    println!("my_function({:?})", i)
}
struct Json;
struct Toml;
struct Cbor;
struct Yaml;

trait Encode {
    fn encode<T: Serialize>(val: T) -> String;
}
impl Encode for Json {
    fn encode<T: Serialize>(val: T) -> String {
        serde_json::to_string(&val).unwrap()
    }
}
impl Encode for Yaml {
    fn encode<T: Serialize>(val: T) -> String {
        serde_yaml::to_string(&val).unwrap()
    }
}
impl Encode for Cbor {
    fn encode<T: Serialize>(val: T) -> String{
        String::from_utf8(serde_cbor::to_vec(&val).unwrap()).unwrap()
    }
}
impl Encode for Toml {
    fn encode<T: Serialize>(val: T) -> String {
        toml::to_string(&val).unwrap()

    }
}

#[derive(Serialize) ]
struct User<T: Encode> {
    name: String,
    age:u32,
    _marker: PhantomData<T>,
}

impl <T> User<T> where T: Encode {
    fn new(name: String, age: u32) -> Self {
        User {
            name,
            age,
            _marker: PhantomData
        }
    }
}



// Acrive: Until Writers Mutable API isused, it is In-Active

///////////////////////////////// main() ///////////////////////
fn main() {
    // let z: impl Fn(i32) -> i32 = |x: i32| x*2;
    let z = |(x, y): (i32, i32)| -> i32 {
        x * y
    };

    let numbers = vec![1,2,3,4,5];
    let handle = thread::spawn(move ||{
        let sum: i32 = numbers.iter().sum();
        println!("the sum is {}", sum)
    });
    // wait for the threads to complete
    handle.join().unwrap();


    let user_json = User::<Json>::new("Alice".to_string(), 32u32);
    let encode = Json::encode(&user_json);
    println!("user_json: {:?}", encode);

    let my_str = "Hello";
    let mut a = 10;

    my_function2(&mut a);
    println!("{}", a);

    my_function( a);
    println!("{}", a);


    // no_param(*my_str); compile error, sized not known
    no_param(1);


    let duck = Duck;
    ducks_say(duck);
    let formal = FormalDuck::new("Ernesto".to_string());
    ducks_say_dyn(&formal);
    ducks_say(formal);

    let mut broken = MyStruct {
        remainder: Some("hello"),
    };
    for _ in 0..5 {
        println!("{:?}", broken.pop_first_char_as_string());
    }

    let hello = Hello;
    (&hello).say_hi();
    (&&hello).say_hi();
    hello.say_hi();

    if let Some(ref mut contents) = Some("hey") {
        println!("{:?}", contents)
    }

    let owned_string: &str = "I am an owned String"; // rust string lieterals are slices
    print_static(owned_string);
    let mut writer = vec![1, 2, 3];
    let reader = &writer;
    println!("len: {}", reader.len());
    writer.push(4);

    let (alice, bob) = ("Alice", "Bob");
    println!("static ref str {}", my_reff());
    println!("static ref str {}", my_reff());
    println!("the longer name is: {}", longest(alice, bob));
    let s: String = String::from("test-ref-string");
    print_mystring(&s);
    print_mystring(&s);
    let s_ref: &String = &s;
    print_mystring(s_ref);
    print_mystring(s_ref);
    'bitcoin_lifteimt: {
        let mut bitcoin = String::from("bitcoin");
        'mut_ref_lifetime: {
            let mut_ref = &mut bitcoin; // borrow bitcoin mutable
            'ro_ref_lifteimt: {
                let ro_ref = &bitcoin;
                println!("{}", ro_ref);
                // mut_ref.push_str("this will fail");
            }
        }
    }
    let (x, y) = (1, 2);
    foobar(&x, &y);

    /*
        let args: Vec<String> = env::args().collect();
        if args.len() != 3 {
            eprintln!("usage {} <op> <text>", args[0]);
            exit(1);
        }
        let op = &args[1];
        let text = &args[2];
        println!("op: {op} text: {text}");
    **/
    println!("end!");
}
