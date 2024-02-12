pub fn remove_first_char(s: &str) -> &str {
    s.char_indices()
        .nth(1) // Find the start of the second character
        .map(|(i, _)| &s[i..]) // Slice from the second character onwards
        .unwrap_or("") // Handle the case where there's 0 or 1 characters
}