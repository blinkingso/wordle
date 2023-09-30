use std::hash::Hash;

///
/// 字母状态
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LetterState {
    // 绿色, 正确的字母
    G = 0,
    // 黄色, 表示字母位置不正确
    Y = 1,
    // 红色, 不正确的字母 或者重复出现的存在的字母
    R = 2,
    // 未知 默认值
    X = 3,
}

/// 字母及字母状态
#[derive(Debug, Clone, Copy)]
pub struct Letter(pub char, pub LetterState);
impl Letter {
    pub fn new(val: char) -> Letter {
        Letter(val, LetterState::X)
    }

    pub fn set_state(&mut self, state: LetterState) {
        self.1 = state;
    }
}

impl PartialOrd for Letter {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Letter {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialEq for Letter {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Letter {}

impl Hash for Letter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    // 测试模式
    #[default]
    Test,
    // 交互式模式
    Interactive,
    // tui模式
    #[cfg(feature = "tui")]
    Tui,
    // gui模式
    #[cfg(feature = "gui")]
    Gui,
}
