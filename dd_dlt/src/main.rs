
use std::{env, io::{self, ErrorKind, Read, Write}, u16};

use zerocopy::{big_endian::U32, AsBytes};
use rapiddlt::dlt_v1::{DltEntry, DltExtendedHeader, DltHTyp, DltStandardHeader, DltStorageEntry, DltStorageHeader};

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("usage: {} <ecu_id> <app_id> <payload_size>", args[0]);
        eprintln!("  reads input form stdin and write valid DLT output with the input as payload to stdout.");

        return Ok(());
    }

    let ecuid = args[1].as_bytes();
    let appid = args[2].as_bytes();

    let payload_size: u16 = match args[3].parse::<u16>() {
        Ok(val) => val,
        Err(_) => panic!("wrong parameter"),
    };

    let mut buf = vec![0u8; payload_size as usize];
    let mut count = 0;

    while true {
        match io::stdin().lock().read_exact(&mut buf) {
            Ok(_) => (),
            _ => {break;}
        };

        // DltStorageHeader
        let sh = DltStorageHeader::new(
            [b'D',b'L',b'T', 0x1],
            count / 100,
            (count % 100) as i32 * 10000,
            ecuid.try_into().expect("ecuid wrong"));

        let len = 4 + 10 + 4 + payload_size;
        let htyp = DltHTyp::new(true, false, false, false, true, 0x1);
        let h = DltStandardHeader::new(htyp, count as u8, len);
        let eh = DltExtendedHeader::new(0, 0, appid.try_into().expect("app id incorrect"), appid.try_into().expect("app id incorrect"));

        io::stdout().write(sh.as_bytes())?;
        io::stdout().write(h.as_bytes())?;
        // TODO ecu_id, session_id
        io::stdout().write(U32::from(count * 100).as_bytes())?;
        io::stdout().write(eh.as_bytes())?;
        io::stdout().write(buf.as_bytes())?;

        count += 1;
    }
    eprintln!("wrote {} DLT messages with payload size={}b", count, payload_size);
    Ok(())
}