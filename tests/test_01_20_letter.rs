use wordle::state::Letter;

#[test]
fn test_letters() {
    let letters = [Letter::new('A'),
        Letter::new('B'),
        Letter::new('C'),
        Letter::new('D'),
        Letter::new('E'),
        Letter::new('F'),
        Letter::new('G')];
    let mut letter = Letter::new('C');
    letter.set_state(wordle::state::LetterState::G);
    assert!(letters.contains(&letter));
}
