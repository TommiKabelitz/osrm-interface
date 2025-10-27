/// Get the (zero-based) index in a string corresponding to a (one-based) line and column
/// number.
///
/// `\n` and `\r\n` are supported newline characters.
#[allow(dead_code)]
pub(crate) fn get_index_of_line_col(s: &str, line: usize, col: usize) -> Option<usize> {
    let mut current_line = 1;
    let mut current_col = 1;

    for (i, ch) in s.char_indices() {
        if current_line == line && current_col == col {
            return Some(i);
        }
        if ch == '\n' {
            current_line += 1;
            current_col = 1;
        } else {
            current_col += 1;
        }
    }

    None
}
