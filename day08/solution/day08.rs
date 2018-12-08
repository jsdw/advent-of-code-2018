macro_rules! err {
    ($($tt:tt)*) => { Box::<std::error::Error>::from(format!($($tt)*)) }
}
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

fn main() -> Result<()> {

    let filename = std::env::args().nth(1).expect("need puzzle input");
    let input: Vec<u8> = std::fs::read_to_string(filename)
        .expect("can't open file")
        .split(char::is_whitespace)
        .filter(|s| !s.is_empty())
        .map(|n| n.parse().expect(&format!("must be number: '{}'", n)))
        .collect();

    let (tree, _) = Tree::from_slice(&input)?;

    println!("Star 1: {}", sum_metadata(&tree));
    println!("Star 2: {}", root_node_value(&tree));

    Ok(())
}

fn sum_metadata(tree: &Tree) -> u32 {
    let mut sum = tree.metadata.iter().fold(0, |acc,&n| acc + n as u32);
    sum += tree.children.iter().fold(0, |acc,c| acc + sum_metadata(c));
    sum
}

fn root_node_value(tree: &Tree) -> u32 {
    if tree.children.len() == 0 {
        return sum_metadata(tree);
    }
    tree.metadata
        .iter()
        .filter(|&&idx| idx != 0)
        .filter_map(|&idx| tree.children.get(idx as usize - 1))
        .fold(0, |acc,c| acc + root_node_value(c))
}

impl Tree {
    fn from_slice(input: &[u8]) -> Result<(Tree,usize)> {
        if input.len() < 2 {
            return Err(err!("slice is not long enough: {:?}", input))
        }

        let child_count = input[0];
        let metadata_count = input[1];

        let mut children = Vec::with_capacity(child_count as usize);
        let mut offset = 2;
        for _ in 0 .. child_count as usize {
            let (child, new_offset) = Tree::from_slice(&input[offset..])?;
            offset = offset + new_offset;
            children.push(child);
        }

        let mut metadata = Vec::with_capacity(metadata_count as usize);
        let last_offset = offset + metadata_count as usize;
        for i in offset .. last_offset {
            let n = *input.get(i).ok_or(err!("metadata not all accounted for"))?;
            metadata.push(n);
        }

        Ok((Tree { children, metadata }, last_offset))
    }
}

struct Tree {
    metadata: Vec<u8>,
    children: Vec<Tree>
}
