fn make_printable(c: u8) -> char {
    if 0x20 <= c && c <= 0x7E {
        c as char
    } else {
        '.'
    }
}

const ROW_LEN: usize = 0x10;
const GROUP_LEN: usize = 8;
const _: () = assert!(ROW_LEN % GROUP_LEN == 0, "GROUP_LEN should divide ROW_LEN");

pub fn print_bytes(b: &[u8]) -> usize {
    let mut off = 0;
    for row in b.chunks(ROW_LEN) {
        // print offset
        print!("{:06X}: ", off);
        off += ROW_LEN;

        // print data table
        let mut i = 0;
        while i < row.len() {
            if i % GROUP_LEN == 0 {
                print!(" ");
            }
            print!("{:02X} ", row[i]);
            i += 1;
        }
        while i < ROW_LEN {
            if i % GROUP_LEN == 0 {
                print!(" ");
            }
            print!("   ");
            i += 1;
        }
        print!(" ");

        // print ASCII representation
        for c in row {
            print!("{}", make_printable(*c));
        }
        println!();
    }
    b.len()
}
