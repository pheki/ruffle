use std::fs::File;
use std::io::BufReader;

fn main() {
    let file = File::open("tests/swfs/SimpleRedBackground.swf").unwrap();
    let reader = BufReader::new(file);
    let (header, data) = swf::read_swf_header(reader).unwrap().read_all().unwrap();
    let swf = swf::read_swf(header, &data).unwrap();
    println!("The SWF has {} frame(s).", swf.header.num_frames);
    println!("The SWF has {} tag(s).", swf.tags.len());
}
