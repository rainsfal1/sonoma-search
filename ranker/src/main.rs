use std::collections::HashSet;

/// Generates a 5x5 matrix for the Playfair cipher based on the given key.
/// The matrix includes all unique letters from the key followed by the
/// remaining letters of the alphabet, omitting 'J' and treating 'I' and 'J' as the same letter.
///
/// # Arguments
///
/// * `key` - A string slice representing the key for matrix generation.
///
/// # Returns
///
/// A 5x5 matrix (`Vec<Vec<char>>`) containing the characters used for encryption/decryption.
fn create_playfair_matrix(key: &str) -> Vec<Vec<char>> {
    let alphabet: Vec<char> = "ABCDEFGHIJKLMNOPRSTUVWXYZ".chars().collect(); // Alphabet without 'J'
    let mut matrix = vec![vec![' '; 5]; 5]; // Initialize 5x5 matrix
    let mut used_chars = HashSet::new(); // Track characters already used in the matrix
    let mut row = 0;
    let mut col = 0;

    // Populate matrix with characters from the key
    for c in key.to_uppercase().chars() {
        if c == 'J' { continue; } // Skip 'J'
        if !used_chars.contains(&c) && c.is_ascii_alphabetic() {
            matrix[row][col] = c;
            used_chars.insert(c);
            col += 1;
            if col == 5 {
                col = 0;
                row += 1;
            }
        }
    }

    // Fill remaining cells with unused alphabet characters
    for &c in &alphabet {
        if !used_chars.contains(&c) {
            matrix[row][col] = c;
            col += 1;
            if col == 5 {
                col = 0;
                row += 1;
            }
        }
    }

    matrix
}

/// Finds the position of a character in the Playfair matrix.
///
/// # Arguments
///
/// * `matrix` - A reference to a 5x5 matrix of characters.
/// * `c` - The character whose position is to be found.
///
/// # Returns
///
/// A tuple containing the row and column of the character within the matrix.
fn find_position(matrix: &Vec<Vec<char>>, c: char) -> (usize, usize) {
    for (i, row) in matrix.iter().enumerate() {
        for (j, &ch) in row.iter().enumerate() {
            if ch == c {
                return (i, j);
            }
        }
    }
    (0, 0) // Default return if character not found (should not occur if matrix is correct)
}

/// Encrypts a pair of characters using the Playfair cipher rules.
///
/// # Arguments
///
/// * `matrix` - A reference to a 5x5 matrix of characters.
/// * `a` - The first character in the pair.
/// * `b` - The second character in the pair.
///
/// # Returns
///
/// A string containing the encrypted pair of characters.
fn encrypt_pair(matrix: &Vec<Vec<char>>, a: char, b: char) -> String {
    let (row_a, col_a) = find_position(matrix, a);
    let (row_b, col_b) = find_position(matrix, b);

    if row_a == row_b {
        // Same row: shift columns to the right
        format!("{}{}", matrix[row_a][(col_a + 1) % 5], matrix[row_b][(col_b + 1) % 5])
    } else if col_a == col_b {
        // Same column: shift rows down
        format!("{}{}", matrix[(row_a + 1) % 5][col_a], matrix[(row_b + 1) % 5][col_b])
    } else {
        // Rectangle rule: swap columns
        format!("{}{}", matrix[row_a][col_b], matrix[row_b][col_a])
    }
}

/// Decrypts a pair of characters using the Playfair cipher rules.
///
/// # Arguments
///
/// * `matrix` - A reference to a 5x5 matrix of characters.
/// * `a` - The first character in the pair.
/// * `b` - The second character in the pair.
///
/// # Returns
///
/// A string containing the decrypted pair of characters.
fn decrypt_pair(matrix: &Vec<Vec<char>>, a: char, b: char) -> String {
    let (row_a, col_a) = find_position(matrix, a);
    let (row_b, col_b) = find_position(matrix, b);

    if row_a == row_b {
        // Same row: shift columns to the left
        format!("{}{}", matrix[row_a][(col_a + 4) % 5], matrix[row_b][(col_b + 4) % 5])
    } else if col_a == col_b {
        // Same column: shift rows up
        format!("{}{}", matrix[(row_a + 4) % 5][col_a], matrix[(row_b + 4) % 5][col_b])
    } else {
        // Rectangle rule: swap columns
        format!("{}{}", matrix[row_a][col_b], matrix[row_b][col_a])
    }
}

/// Encrypts a plaintext string using the Playfair cipher.
/// The function handles repeated letters, appending 'Q' between them and at the end if necessary.
///
/// # Arguments
///
/// * `plaintext` - The plaintext message to be encrypted.
/// * `key` - The key for matrix generation.
///
/// # Returns
///
/// A string containing the encrypted ciphertext.
fn playfair_encrypt(plaintext: &str, key: &str) -> String {
    let matrix = create_playfair_matrix(key);
    let mut ciphertext = String::new();
    // Filter only alphabetic characters and convert to uppercase
    let mut chars: Vec<char> = plaintext.to_uppercase().chars().filter(|c| c.is_ascii_alphabetic()).collect();

    // If the length of characters is odd, append a 'X'
    if chars.len() % 2 != 0 {
        chars.push('X');
    }

    let mut i = 0;
    while i < chars.len() {
        let a = chars[i];
        let mut b = chars[i + 1];

        if a == b {
            b = 'X'; // If both characters in the pair are the same, replace the second with 'X'
            i -= 1;  // Reprocess the current character
        }

        ciphertext.push_str(&encrypt_pair(&matrix, a, b));
        i += 2;
    }

    ciphertext
}

/// Decrypts a ciphertext string encrypted using the Playfair cipher.
///
/// # Arguments
///
/// * `ciphertext` - The ciphertext to be decrypted.
/// * `key` - The key for matrix generation.
///
/// # Returns
///
/// A string containing the decrypted plaintext.
fn playfair_decrypt(ciphertext: &str, key: &str) -> String {
    let matrix = create_playfair_matrix(key);
    let mut plaintext = String::new();
    let chars: Vec<char> = ciphertext.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let a = chars[i];
        let b = chars[i + 1];
        plaintext.push_str(&decrypt_pair(&matrix, a, b));
        i += 2;
    }

    plaintext
}

fn main() {
    let plaintext = "ELEPHANT";
    let key = "A DIFFEERENT EXAMPLE";

    let encrypted = playfair_encrypt(plaintext, key);
    println!("Plaintext: {}", plaintext);
    println!("Encrypted: {}", encrypted);

    let decrypted = playfair_decrypt(&encrypted, key);
    println!("Decrypted: {}", decrypted);
}
