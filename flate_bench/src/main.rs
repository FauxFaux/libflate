extern crate clap;
extern crate flate2;
extern crate libflate;

use std::fs;
use std::io;
use std::io::Read;
use std::time;
use clap::App;
use clap::Arg;

fn main() {
    let matches = App::new("flate_bench")
        .arg(Arg::with_name("INPUT")
            .index(1)
            .required(true))
        .arg(Arg::with_name("DISABLE_FLATE2").long("disable-flate2"))
        .arg(Arg::with_name("DISABLE_LIBFLATE").long("disable-libflate"))
        .get_matches();

    let input_file_path = matches.value_of("INPUT").unwrap();
    let mut plain = Vec::new();
    fs::File::open(input_file_path).unwrap().read_to_end(&mut plain).unwrap();

    println!("");
    println!("# ENCODE (input_size={})", plain.len());
    if !matches.is_present("DISABLE_LIBFLATE") {
        bench("- libflate",
              io::Cursor::new(&plain),
              libflate::deflate::Encoder::new(BenchWriter::new()));
    }
    if !matches.is_present("DISABLE_FLATE2") {
        bench("-   flate2",
              io::Cursor::new(&plain),
              flate2::write::DeflateEncoder::new(BenchWriter::new(), flate2::Compression::Default));
    }
    println!("");

    let compressed = {
        let mut input_file = fs::File::open(input_file_path).unwrap();
        let mut writer = flate2::write::DeflateEncoder::new(Vec::new(),
                                                            flate2::Compression::Default);
        io::copy(&mut input_file, &mut writer).unwrap();
        writer.finish().unwrap()
    };
    println!("# DECODE (input_size={})", compressed.len());
    if !matches.is_present("DISABLE_LIBFLATE") {
        bench("- libflate",
              libflate::deflate::Decoder::new(io::Cursor::new(&compressed)),
              BenchWriter::new());
    }
    if !matches.is_present("DISABLE_FLATE2") {
        bench("-   flate2",
              flate2::read::DeflateDecoder::new(io::Cursor::new(&compressed)),
              BenchWriter::new());
    }
    println!("");
}

fn bench<R, W>(tag: &str, mut reader: R, mut writer: W)
    where R: io::Read,
          W: io::Write + Into<BenchWriter>
{
    io::copy(&mut reader, &mut writer).unwrap();
    let (elapsed, size) = writer.into().finish();
    println!("{}: elapsed={}.{:06}s, size={}",
             tag,
             elapsed.as_secs(),
             elapsed.subsec_nanos() / 1000,
             size);
}

struct BenchWriter {
    started_at: time::Instant,
    written_size: u64,
}
impl BenchWriter {
    pub fn new() -> Self {
        BenchWriter {
            started_at: time::Instant::now(),
            written_size: 0,
        }
    }
    pub fn finish(self) -> (time::Duration, u64) {
        (self.started_at.elapsed(), self.written_size)
    }
}
impl io::Write for BenchWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.written_size += buf.len() as u64;
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl From<libflate::deflate::Encoder<BenchWriter>> for BenchWriter {
    fn from(f: libflate::deflate::Encoder<BenchWriter>) -> Self {
        f.finish().into_result().unwrap()
    }
}
impl From<flate2::write::DeflateEncoder<BenchWriter>> for BenchWriter {
    fn from(f: flate2::write::DeflateEncoder<BenchWriter>) -> Self {
        f.finish().unwrap()
    }
}