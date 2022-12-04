pub fn split_string(str: Vec<u8>) -> Vec<u8> {
    let mut final_str: Vec<u8> = Vec::new();

    let mut position = 0;
    while position < str.len() {
        if str[position] == b'\\' {
            position += 1;
            match str.get(position) {
                None => break,
                Some(b'a') => final_str.push(b'\x07'),
                Some(b'b') => final_str.push(b'\x08'),
                Some(b't') => final_str.push(b'\t'),
                Some(b'n') => final_str.push(b'\n'),
                Some(b'v') => final_str.push(b'\x0b'),
                Some(b'f') => final_str.push(b'\x0c'),
                Some(b'r') => final_str.push(b'\r'),
                Some(b' ') => final_str.push(b' '),
                Some(b'\\') => final_str.push(b'\\'),
                Some(b'x') => {
                    while str.get(position).unwrap_or(&b'\0').is_ascii_hexdigit() {
                        final_str.push(str[position])
                    }
                }
                Some(a) => final_str.push(*a),
            }
        } else {
            final_str.push(str[position]);
        }
        position += 1;
    }
    final_str
}
