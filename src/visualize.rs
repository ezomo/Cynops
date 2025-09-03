pub trait Visualize {
    fn visualize(&self);
    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]);
}

// Helper functions for tree visualization
pub fn print_branch(label: &str, value: &str, _indent: usize, is_last: bool, prefix: &[bool]) {
    // Draw the tree structure
    for &p in prefix {
        if p {
            print!("│   ");
        } else {
            print!("    ");
        }
    }

    // Draw the current branch
    if is_last {
        print!("└── ");
    } else {
        print!("├── ");
    }

    // Print the label and value
    if value.is_empty() {
        println!("{}", label);
    } else {
        println!("{}: {}", label, value);
    }
}

pub fn extend_prefix(prefix: &[bool], has_more: bool) -> Vec<bool> {
    let mut new = prefix.to_vec();
    new.push(has_more);
    new
}

pub trait OneLine {
    fn oneline(&self) -> String;
}
