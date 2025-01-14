use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    // 读取文件内容
    // 本地文件直接可以使用env传参
    let args: Vec<String> = env::args().collect();
    println!("args: {:?}", args);

    let filename = &args[1];
    let mut f = File::open(filename).unwrap();
    let mut contents = String::new();

    f.read_to_string(&mut contents).unwrap();
    let mut sum = 0;
    for c in contents.lines() {
        let n = c.parse::<i32>().unwrap();
        sum += n;
    }

    println!("{:?}", sum);
}
