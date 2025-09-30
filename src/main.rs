mod fpga;

fn main() {
    //let args: Vec<String> = env::args().collect();
    //dbg!(args);

    let mut i = 0;
    let mut dir: i8 = 1;

    let width = 5;
    let height = 10;

    let mut j = 0;
    for _ in 0..height * (width) {
        println!("{}, {}", i, j);

        if (i == width - 1 && dir == 1) || (i == 0 && dir == -1) {
            println!("switch");
            dir *= -1;
            j += 1;
        } else {
            i = (i as isize + dir as isize) as usize;
        }
    }
}
