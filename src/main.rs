use std::collections::{HashMap, BinaryHeap};
use std::io::{Read, Write, BufReader, stdin, self};

fn main() -> Result<(), io::Error> {
    let map = parse()?;

    #[cfg(not)]
    println!("Map: {:#?}", map);

    let tree = Tree::from(map);

    #[cfg(not)]
    println!("Tree: {:#?}", tree);

    let mut encode: Vec<_> = tree.encode().into_iter().collect();
    encode.sort_by(|(_, (l_code, l_depth)), (_, (r_code, r_depth))| {
        if l_depth < r_depth {
            (l_code << (r_depth - l_depth)).cmp(r_code)
        } else {
            (l_code).cmp(&(r_code << (l_depth - r_depth)))
        }
    });

    println!("Encoding");
    println!("========");
    for (c, (code, depth)) in encode {
        println!(
            "{0:4} => {1:>#02$b}",
            format!("{:?}", std::char::from_u32(c as u32).expect("Invalid ASCII character")),
            code, depth + 2
        );
    }

    Ok(())
}

fn parse() -> Result<HashMap<u8, u64>, io::Error> {
    let mut map = HashMap::new();

    let stdin = BufReader::with_capacity(1 << 16, stdin());
    for c in stdin.bytes() {
        let c = c?;
        let seen = map.remove(&c).unwrap_or(0);
        map.insert(c, seen + 1);
    }

    Ok(map)
}

#[derive(Debug, Ord, Eq, PartialEq)]
enum Tree {
    Leaf(u8, u64),
    Node(Box<Tree>, Box<Tree>, u64),
}
use self::Tree::*;

impl Tree {
    fn prob(&self) -> u64 {
        match self {
            Leaf(_, p) => *p,
            Node(_, _, p) => *p,
        }
    }

    fn encode(&self) -> HashMap<u8, (u64, usize)> {
        fn recurse(node: &Tree, map: &mut HashMap<u8, (u64, usize)>, prefix: u64, depth: usize) {
            match node {
                Leaf(c, _) => {
                    map.insert(*c, (prefix, depth));
                }
                Node(l, r, _) => {
                    recurse(&l, map, (prefix << 1) | 0, depth + 1);
                    recurse(&r, map, (prefix << 1) | 1, depth + 1);
                }
            }
        }

        let mut map = HashMap::new();
        recurse(self, &mut map, 0, 0);
        map
    }
}

impl std::ops::Add for Tree {
    type Output = Self;

    fn add(self: Tree, right: Tree) -> Tree {
        let total_prob = self.prob() + right.prob();
        Node(Box::new(self), Box::new(right), total_prob)
    }
}

impl std::cmp::PartialOrd for Tree {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(other.prob().cmp(&self.prob()))
    }
}

impl From<HashMap<u8, u64>> for Tree {
    fn from(probs: HashMap<u8, u64>) -> Tree {
        let mut queue: BinaryHeap<_> = probs.into_iter()
            .map(|(c, count)| Leaf(c, count))
            .collect();

        while queue.len() > 1 {
            let first = queue.pop().unwrap();
            let second = queue.pop().unwrap();
            queue.push(first + second)
        }

        queue.pop().expect("At least one character")
    }
}

/// Write individual bits to a file. Least significant bits first.
struct BitWriter<W: Write> {
    buffer: u8,
    buffer_len: usize,
    inner: W,
}

impl<W: Write> BitWriter<W> {
    const BYTE_BITS: usize = 8;

    fn new(inner: W) -> BitWriter<W> {
        BitWriter { buffer: 0u8, buffer_len: 0usize, inner }
    }

    fn write_bits(&mut self, bits: u64, length: usize) -> Result<(), io::Error> {
        let mut pair = (bits, length);
        while pair.1 > 0usize {
            pair = self.consume_bits(pair);
            self.flush_byte()?;
        }

        Ok(())
    }

    fn flush_byte(&mut self) -> Result <(), io::Error> {
        if self.buffer_len == Self::BYTE_BITS {
            let byte = [self.buffer];
            self.inner.write(&byte)?;
            self.buffer_len = 0;
        }

        Ok(())
    }

    fn consume_bits(&mut self, (bits, length): (u64, usize)) -> (u64, usize) {
        let to_consume = Self::BYTE_BITS.saturating_sub(self.buffer_len).min(length);
        self.buffer = self.buffer.overflowing_shl(to_consume as u32).0;
        self.buffer |= (bits as u8) & ((1 << to_consume) - 1);
        (bits.overflowing_shr(to_consume as u32).0, length - to_consume)
    }
}

impl<W: Write> Drop for BitWriter<W> {
    fn drop(&mut self) {
        if self.buffer_len > 0 {
            let byte = [self.buffer];
            self.inner.write(&byte).expect("Flush final byte");
        }
    }
}
