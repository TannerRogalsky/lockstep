use serde::{Deserialize, Serialize};

pub const INPUT_BUFFER_FRAMES: super::FrameIndex = 7;
type Input = super::IndexedState<super::AddBodyEvent>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct OrderedInput(Input);
impl std::cmp::PartialEq for OrderedInput {
    fn eq(&self, other: &Self) -> bool {
        self.0.frame_index.eq(&other.0.frame_index)
    }
}
impl std::cmp::Eq for OrderedInput {}
impl std::cmp::PartialOrd for OrderedInput {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.frame_index.partial_cmp(&other.0.frame_index)
    }
}
impl std::cmp::Ord for OrderedInput {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.frame_index.cmp(&other.0.frame_index)
    }
}
impl std::hash::Hash for OrderedInput {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.frame_index.hash(state)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputBuffer(std::collections::BinaryHeap<OrderedInput>);

impl InputBuffer {
    pub fn push(&mut self, input: Input) {
        self.0.push(OrderedInput(input))
    }

    pub fn next(&mut self, index: super::FrameIndex) -> Option<Input> {
        match self.0.peek() {
            None => None,
            Some(input) => {
                if input.0.frame_index <= index {
                    let input = self.0.pop().unwrap();
                    Some(input.0)
                } else {
                    None
                }
            }
        }
    }
}
