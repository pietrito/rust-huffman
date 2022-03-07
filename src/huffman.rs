use huffman::file_bin;

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::io::Write;
use std::io::Seek;
use std::io::SeekFrom;
use std::vec::Vec;

/// Size of the chunks while reading input files
const CHUNK_SIZE: usize = 4096;
/// Self explanatory
pub static VERBOSE: bool = true;

// Tree Node: Contains a byte and possible (e.g Options) leafs
#[derive (Eq)]
pub struct TNode {
    byte: Option<u8>,
    left: Option<Box<TNode>>,
    right: Option<Box<TNode>>,
}

/// Node of a Huffman Tree
impl TNode {
    // Creates a Leaf Node from contained byte
    pub fn new(byte: u8) -> Self {
        TNode {
            byte: Some(byte),
            left: None,
            right: None,
        }
    }

    // Creates a Branch Node from 2 'leaf' node
    pub fn new_branch(left: Option<Box<TNode>>, right: Option<Box<TNode>>) -> Self {
        TNode {
            byte: None,
            left,
            right
        }
    }
}

// Trait for println!("{}", TNode)
impl fmt::Display for TNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.byte {
            Some(byte) => write!(f, "b: {}", byte),
            None => write!(f, "b: Ø")
        }
    }
}

// Trait for TNode1 == TNode2
impl PartialEq for TNode {
    fn eq(&self, other: &Self) -> bool {
        self.byte == other.byte
    }
}

// List Node:
// - w: usize -> weight of this node (e.g number of occurences it actually represents)
// - next: Option<LNode> -> if not null, next list node
#[derive (Eq)]
pub struct LNode {
    weight: usize,
    tree_node: Option<Box<TNode>>,
}

/// A Huffman List node
impl LNode {
    // Creates a List Node with a weight and contained Tree Node
    fn new(weight: usize, tree_node: Option<Box<TNode>>) -> Self {
        LNode {
            weight,
            tree_node
        }
    }
}

// Trait for LNode1 == LNode2
impl PartialEq for LNode {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}

// Trait for LNode1 < LNode2
impl Ord for LNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.weight.cmp(&other.weight)
    }
}

// Trait for LNode1 <= LNode2
impl PartialOrd for LNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Implement display trait for List Node
impl fmt::Display for LNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.tree_node.as_ref().unwrap().byte {
            Some(byte) => write!(f, "w: {} - c: {}", self.weight, byte),
            None => write!(f, "w: {} - c: Ø", self.weight)
        }
    }
}

/// Main compress() function
pub fn compress(path_in: &String, path_out: &String) -> Result<(), Box<dyn Error>> {

    // TODO: Move this arg logic to main.rs
    println!("[+] HUFFMAN COMPRESS");
    println!("[+] In file: [{}]", path_in);
    println!("[+] Out file: [{}]", path_out);

    // Open the file and use a bufreader for it
    let file_in = std::fs::File::open(path_in)?;

    // Get expected List length
    let nb = count_diff_chars(&file_in)?;
    println!("[+] Number of different bytes: {}", nb);

    // Huffman List
    let mut huffman_list = build_list(&file_in)?;
    if huffman_list.len() == 0 {
        panic!("[-] Cannot compress an empty file.");
    } else if huffman_list.len() != nb {
        panic!("[-] There was an error while building the Huffman List.");
    } else {
        println!("[+] Successfully built Huffman List for [{}] different characters.", huffman_list.len());
    }

    // Huffman Tree
    let (huffman_tree, file_size) = build_tree(&mut huffman_list)?;
    if file_size as u64 != file_in.metadata().unwrap().len() {
        panic!("[-] There was an error while building the Huffman Tree.");
    } else {
        println!("[+] Successfully built tree for [{}] bytes", file_size);
    }

    // Huffman Codes
    let mut huffman_codes = [None; 256];
    gen_codes(Some(&huffman_tree), [None; 30], 0, &mut huffman_codes);

    // Binary output file
    let mut file_out = file_bin::BinFile::create(&path_out)?;
    if VERBOSE {
        println!("[=] [{}] [SIZE]> {} bytes", path_out, file_size);
    }
    // Write header (original file size) and Huffman Tree
    file_out.write_bytes(&file_size.to_le_bytes())?;
    println!("[+] Successfully wrote header.");
    if VERBOSE { print!("[=] [{}] [TREE]> ", path_out); }
    write_tree(&mut file_out, Some(&huffman_tree))?;
    println!();
    println!("[+] Successfully wrote tree.");
    // Write the compressed data
    compress_file(&file_in, &mut file_out, &huffman_codes)?;
    file_out.flush()?;
    println!();
    println!("[+] Finished writing compressed file.");

    Ok(())
}

/// Writes the given Huffman tree to the given file.
/// ⚠ This function is recursive.
///
/// # Arguments
///
/// * `binfile` - A huffman::binfile structure reference. See its documentation for more information.
/// * `tree` - The Huffman TNode head to consider.
///
/// Note: This function should usually be called on the head of a Huffman Tree.
///
pub fn write_tree(binfile: &mut file_bin::BinFile, tree: Option<&TNode>) -> Result<(), Box<dyn Error>> {

    match tree {
        None => Ok(()),
        Some(node) => {
            if node.byte.is_some() {
                if VERBOSE { print!("0{}",node.byte.unwrap()); }
                binfile.write_bit(false)?;
                binfile.write_byte(node.byte.unwrap())?;
            } else {
                if VERBOSE { print!("1"); }
                binfile.write_bit(true)?;
                write_tree(binfile, Some(&*node.left.as_ref().unwrap()))?;
                write_tree(binfile, Some(&*node.right.as_ref().unwrap()))?;
            }

            Ok(())
        }
    }
}

pub fn read_tree(binfile: &mut file_bin::BinFile) -> Result<Option<Box<TNode>>, Box<dyn Error>> {
    match binfile.read_bit()? {
        true => {
            if VERBOSE { print!("1"); }
            Ok(Some(Box::new(TNode::new_branch(read_tree(binfile)?, read_tree(binfile)?))))
        },
        false => {
            if VERBOSE { print!("0"); }
            let byte = binfile.read_byte()?;
            if VERBOSE { print!("{}", byte); }
            Ok(Some(Box::new(TNode::new(byte))))
        }
    }
}

pub fn compress_file(in_file: &std::fs::File, out_bin_file: &mut file_bin::BinFile, codes: &[Option<[Option<bool>;30]>; 256]) -> Result<(), Box<dyn Error>> {

    let mut total = 0;
    loop {
        let mut chunk = Vec::with_capacity(CHUNK_SIZE);
        let n = in_file.take(CHUNK_SIZE as u64).read_to_end(&mut chunk)?;
        if n == 0 { break; }
        for x in chunk {
            if codes[x as usize].is_some() {
                for i in 0usize..30usize {
                    if codes[x as usize].unwrap()[i].is_some() {
                        let r = out_bin_file.write_bit(codes[x as usize].unwrap()[i].unwrap())?;
                        if r {
                            total += 1;
                            if VERBOSE { print!("\r[=] [{}] [BYTES]> {}", out_bin_file.path, total); }
                        }
                    } else {
                        break;
                    }
                }
           } else {
               panic!("[-] Char codes missing.");
           }
        }
        if n < CHUNK_SIZE { break; }
    }


    Ok(())
}

pub fn count_diff_chars(mut file: &std::fs::File) -> Result<usize, Box<dyn Error>> {
    let mut ret: usize = 0;
    let mut found = Vec::new();

    loop {
        let mut chunk = Vec::with_capacity(CHUNK_SIZE);
        let n = file.take(CHUNK_SIZE as u64).read_to_end(&mut chunk)?;
        if n == 0 { break; }
        for x in chunk {
            if !found.contains(&x) {
                found.push(x);
                ret += 1;
                if ret == 256 { break; }
            }
        }
        if ret == 256 { break; }
        if n < CHUNK_SIZE { break; }
    }

    file.seek(SeekFrom::Start(0))?;

    Ok(ret)
}

fn build_list(mut file: &std::fs::File) -> Result<Vec<LNode>, Box<dyn Error>> {
    // Count array: index is the byte value and value is the count of this byte
    let mut count = [0usize; 256];
    // Read the file chunk by chunk and increment the count array
    loop {
        let mut chunk = Vec::with_capacity(CHUNK_SIZE);
        let n = file.take(CHUNK_SIZE as u64).read_to_end(&mut chunk)?;
        if n == 0 { break; }
        for x in chunk {
            count[x as usize] += 1;
        }
        if n < CHUNK_SIZE { break; }
    }

    // Reset the file pointer to its begining for later
    file.seek(SeekFrom::Start(0))?;

    // The return vector containing List Nodes
    let mut ret = Vec::with_capacity(count_diff_chars(file)?);
    // Loop thru the previously built array
    loop {
        // Get the current minimum value
        let mut current_min = std::usize::MAX;
        let mut current_min_i = 0usize;
        for i in 0..256 {
            if count[i] != 0 {
                if count[i] < current_min {
                    current_min = count[i];
                    current_min_i = i;
                }
            }
        }

        // If we got one, push it at the begining of the list
        if current_min != std::usize::MAX
        {
            // New empty-leafed Tree Node containing the byte
            let new_tnode = TNode::new(current_min_i as u8);
            // New list item containing the Tree Node
            let new_node = LNode::new(current_min, Some(Box::new(new_tnode)));
            // Append the new node to the vector
            ret.push(new_node);
            // Remove this value from the list
            count[current_min_i] = 0;
        // If we didn't get any new byte, break out of the loop
        } else { break; }
    }

    // Return the vector containing the List Nodes
    Ok(ret)
}

pub fn build_tree(vec: &mut Vec<LNode>) -> Result<(TNode, usize), Box<dyn Error>> {
    // Check for empty Huffman list
    if vec.len() == 0 {
        panic!("[-] Need a non empty list to build a tree.");
    }

    // Loop while we have more than a single node in the Huffman List
    while vec.len() > 1 {
        // Get the 2 first nodes (lowest char counts)
        let first = vec.remove(0);
        let second = vec.remove(0);
        // Extract the data from the first 2 nodes
        let new_weight = first.weight + second.weight;
        let first_tnode = first.tree_node;
        let second_tnode = second.tree_node;
        // Create a new Tree Node with its 2 leafs being the first 2 nodes of the list
        let new_tnode = TNode::new_branch(first_tnode, second_tnode);
        // Create a new List Node containing the new Tree Node
        let new_lnode: LNode = LNode::new(new_weight, Some(Box::new(new_tnode)));
        // Iterate through the list to get the new node's index
        let mut i = 0;
        let index = loop {
            if i == vec.len() || new_weight < vec[i].weight {
                break i;
            }
            i += 1;
        };
        // Insert the node at its index
        vec.insert(index, new_lnode);
    }

    // Return the Tree head Node and the sum of all weight
    let head = vec.pop().unwrap();
    Ok((*head.tree_node.unwrap(), head.weight))
}

/// Generates binary codes for each byte present in the given Huffman Tree by going through it
pub fn gen_codes(tree: Option<&TNode>, mut current: [Option<bool>;30], current_index: usize, codes: &mut [Option<[Option<bool>; 30]>; 256]) {
    match tree {
        Some(tnode) => {
            match tnode.byte {
                Some(byte) => {
                    codes[byte as usize] = Some(current);
                    return;
                },
                None => {
                    current[current_index] = Some(false);
                    gen_codes(Some(&*tnode.left.as_ref().unwrap()), current, current_index + 1, codes);
                    current[current_index] = Some(true);
                    gen_codes(Some(&*tnode.right.as_ref().unwrap()), current, current_index + 1, codes);
                },
            }
        }
        None => return,
    }
}

/// This is the main decompress function
pub fn decompress(path_in: &String, path_out: &String) -> Result<(), Box<dyn Error>> {
    // Open the file and use a bufreader for it
    let mut file_in = file_bin::BinFile::open(path_in)?;
    // Read the first 8bytes: size
    let nb = file_in.read_size()?;
    if VERBOSE { println!("[=] [{}][DEC]> {} bytes", path_in, nb); }
    // Read and build the Huffman Tree
    if VERBOSE { print!("[=] [{}] [TREE]> ", path_out); }
    let tree = read_tree(&mut file_in)?;
    if VERBOSE { println!(); }
    // Read the compressed data and write the decompressed data in the output file
    let mut file_out = std::fs::File::create(&path_out)?;
    let n = decompress_file(&mut file_in, Some(&tree.unwrap()), &mut file_out)?;
    if VERBOSE { println!(); }
    println!("[+] Decompressed [{}] bytes.", n);

    Ok(())
}

/// This the function that actually performs the Huffman decompression
pub fn decompress_file(binfile: &mut file_bin::BinFile, tree: Option<&TNode>, out_file: &mut std::fs::File) -> Result<usize, Box<dyn Error>> {
    // Returned number of bytes decompressed
    let mut ret = 0;
    // Check if we already read the size from the binary file
    if binfile.size.is_none() {
        panic!("[-] Size must be read before.");
    }

    // Loop until we've read the expected number of bytes
    while ret != binfile.size.unwrap() {
        // Read and decompress a single byte
        let byte = decompress_byte(binfile, tree)?;
        // Write it to the output file
        out_file.write_all(&[byte])?;
        if VERBOSE { print!("\r[=] [{}] [BYTES]> {}", binfile.path, ret); }
        // Increment the byte count
        ret += 1;
    }

    Ok(ret)
}

/// Recursive function that reads a single byte from the given binary file
pub fn decompress_byte(binfile: &mut file_bin::BinFile, node: Option<&TNode>) -> Result<u8, Box<dyn Error>> {
    if node.unwrap().byte.is_some() {
        return Ok(node.unwrap().byte.unwrap());
    }

    let bit = binfile.read_bit()?;
    if !bit {
        return decompress_byte(binfile, Some(&*node.unwrap().left.as_ref().unwrap()));
    }

    return decompress_byte(binfile, Some(&*node.unwrap().right.as_ref().unwrap()));
}

/// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_nb_diff_chars() -> Result<(), Box<dyn Error>> {
        let file_lazydog = std::fs::File::open("./data/lazy_dog.txt")?;
        let file_tara = std::fs::File::open("./data/tara.txt")?;
        let file_empty = std::fs::File::open("./data/empty.txt")?;

        assert_eq!(count_diff_chars(&file_lazydog)?, 29);
        assert_eq!(count_diff_chars(&file_tara)?, 5);
        assert_eq!(count_diff_chars(&file_empty)?, 0);

        Ok(())
    }

    #[test]
    fn check_build_list() -> Result<(), Box<dyn Error>> {
        let file_lazydog = std::fs::File::open("./data/lazy_dog.txt")?;
        let file_tara = std::fs::File::open("./data/tara.txt")?;
        let file_empty = std::fs::File::open("./data/empty.txt")?;

        assert_eq!(build_list(&file_lazydog)?.len(), count_diff_chars(&file_lazydog)?);
        assert_eq!(build_list(&file_tara)?.len(), count_diff_chars(&file_tara)?);
        assert_eq!(build_list(&file_empty)?.len(), count_diff_chars(&file_empty)?);

        Ok(())
    }

    #[test]
    fn check_build_tree() -> Result<(), Box<dyn Error>> {
        let file_lazydog = std::fs::File::open("./data/lazy_dog.txt")?;
        let file_tara = std::fs::File::open("./data/tara.txt")?;

        let mut list_lazydog = build_list(&file_lazydog)?;
        let mut list_tara = build_list(&file_tara)?;

        let (_, count_lazydog) = build_tree(&mut list_lazydog)?;
        let (_, count_tara) = build_tree(&mut list_tara)?;

        assert_eq!(count_lazydog as u64, file_lazydog.metadata().unwrap().len());
        assert_eq!(count_tara as u64, file_tara.metadata().unwrap().len());

        Ok(())
    }

    #[test]
    #[should_panic]
    fn check_empty_file() {
        let file_empty = match std::fs::File::open("./data/empty.txt") {
            Err(err) => panic!("{:?}", err),
            Ok(file) => file,
        };

        let mut list_empty = match build_list(&file_empty) {
            Err(err) => panic!("{:?}", err),
            Ok(list) => list,
        };

        let (_,_) = match build_tree(&mut list_empty) {
            Ok((t, w)) => (t,w),
            Err(err) => panic!("{:?}", err)
        };
    }
}