use super::*;
use std::collections::HashMap;
use std::io::{self, Read};

pub fn exec_bf(bf: &[BF]) {
    let fast = bf.iter().cloned().map(FastBF::from).collect::<Vec<_>>();

    let map = parse_bracs(&fast);
    let mut ip = 0;
    let mut stack = vec![0 as u16];
    let mut head = 0;

    while ip < bf.len() {
        use BF::*;
        match bf[ip] {
            Profile(_) => (),
            Dbg(_msg) => {
                #[cfg(feature = "debugbf")]
                {
                    dbg!(head);
                    dbg!(_msg);
                }
            }
            Left => head -= 1,
            Right => {
                head += 1;
                if stack.len() == head {
                    stack.push(0);
                }
            }
            Inc => stack[head] = stack[head].wrapping_add(1),
            Dec => stack[head] = stack[head].wrapping_sub(1),
            Input => stack[head] = io::stdin().bytes().next().unwrap().unwrap() as u16,
            Output => output(stack[head] as u8),
            LBrac => {
                if stack[head] == 0 {
                    ip = map[&ip];
                }
            }
            RBrac => {
                if stack[head] != 0 {
                    ip = map[&ip];
                }
            }
        }
        ip += 1;
    }
}

fn parse_bracs(code: &[FastBF]) -> HashMap<usize, usize> {
    let mut map = HashMap::default();
    let mut bracs = vec![];

    for (i, inst) in code.iter().enumerate() {
        use FastBF::*;
        match inst {
            LB => bracs.push(i),
            RB => {
                let start = bracs.pop().unwrap();
                map.insert(i, start);
                map.insert(start, i);
            }
            _ => (),
        }
    }

    map
}

fn output(b: u8) {
    use std::io::{self, Write};
    let mut stderr = io::stderr();
    stderr.write_all(&[b]).unwrap();
    stderr.flush().unwrap();
}
